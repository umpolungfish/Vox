---
name: gpu_native_gmomonados
description: "The port making G-mOMonadOS a real GPU machine (every op on the card, host a dumb loader), its plan, and the WSL2 mailbox constraint that shapes it"
metadata: 
  node_type: memory
  type: project
  originSessionId: 385edc78-5c2b-4d2e-a776-32d23a7e2a97
  modified: 2026-09-15T11:06:39.833Z
---

The build target Lando set: when G-mOMonadOS boots, not a single operation runs
on the CPU. Every command and operation ported to the GPU, and everything
self-contained with no linking across repos.

**Plan** lives at `~/.claude/plans/no-you-plan-out-virtual-hare.md`. Realization:
one resident megakernel launched once that never exits, holding OS state in
device memory and running a command loop on the card; the host becomes a dumb
loader and mailbox pump (upload once, launch once, then only write command / read
result / print). The launch-and-shuttle floor is named plainly, not pretended to
be zero. Phased: sever cross-repo deps and freeze a CPU control battery, then
the megakernel spine, then boot compute onto the card, then command dispatch by
engine group, then delete CPU compute paths, then verify against the frozen
control.

**Done so far** (branch main, G-mOMonadOS its own repo):
- `db2ed2c` vendored imasm_core (was ../MoDoT/imasm_core) and vox (was ../Vox)
  into `vendor/`, repointed path deps, build green, cross-repo links severed.
  Control battery frozen at `gpu_port/control/`.
- `d471489` proved the resident-megakernel mailbox mechanism in
  `gpu_port/spine_probe/`.

**The constraint that shapes the spine:** both cards (RTX 4070 device 0, RTX 3060
device 1) report `concurrent_managed_access=0` under WSL2. So unified/managed
memory (`cuMemAllocManaged`) CANNOT back the host<->device mailbox; the host
touching managed memory while a kernel is resident segfaults. The mechanism that
works is mapped pinned host memory: `result::malloc_host(bytes,
CU_MEMHOSTALLOC_DEVICEMAP|CU_MEMHOSTALLOC_PORTABLE)` then
`cuMemHostGetDevicePointer_v2`; the host owns the pinned buffer and polls a
doorbell with volatile read/write, the device sees it mapped zero-copy and polls
it, the kernel uses `__threadfence_system()` around the doorbell. No concurrent
managed access required. Zero-copy polling crosses PCIe so it is slow: fine for
low-frequency command dispatch, wrong for the compute itself, which must run in
device memory.

Native Windows was checked and ruled out (a Windows-native executor driven from
WSL was Lando's idea to escape the WSL passthrough). A cross-built Windows CUDA
exe run from WSL reported `concurrent_managed_access=0` on BOTH cards on the
native driver too: it is the WDDM consumer-GeForce driver model, not WSL, so
Windows gives no unified-memory gain. Decision: WSL only, no second target. The
cross-build path itself works trivially (x86_64-pc-windows-gnu + mingw-w64, WSL
runs the .exe directly against nvcuda.dll) if a Windows target is ever wanted for
other reasons. TCC mode on the compute card is the only untried route to real
concurrent managed access.

CUDA binding is `cudarc = "=0.19.9"` with features driver, nvrtc, cuda-12040,
dynamic-loading, std. `cudarc::driver::result` and `::sys` are public. See
[[reference_ground_truth_is_p4rakernel]] for the kernel ground truth, and
[[feedback_build_in_vox_not_rust]] for the standing pull to build in Vox.
