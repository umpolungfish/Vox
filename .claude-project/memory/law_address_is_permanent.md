---
name: law-address-is-permanent
description: "An enumerated crystal address is never removed; the imscription at it is freely revisable, and the lattice rejects improper imscriptions by its own coupling."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: e808a460-7d8e-4969-9d96-44a24695b03e
  modified: 2026-09-01T05:34:38.491Z
---

Once an address in the crystal has been **enumerated**, it must not be removed
under any circumstances. A valid address is a valid address, simple as.

The **imscription** — the outward form assigned to that address — is **not**
bound by that restriction. An imscription may be revised, replaced, or disputed
freely. One's particular imscription of an ixcription, and vice versa, is a
matter of **testing a composition**, not of adjudicating a defect.

The lattice is universally coupled to itself, so it **rejects improper
imscriptions inherently**. No external validator is needed to police them, and
none should be built.

**Why:** the address space is the invariant the whole crystal rests on; deleting
an address destroys the ground other addresses are measured against. The form
carried at an address is a claim about it, and claims are tested by composing
them, which the coupling already does.

**How to apply:**
- Never delete, dedup-away, or "clean up" an enumerated entry. Record instead —
  a merge that drops a repeat must keep what it dropped. dS = 0: hold the
  contradiction, never discard it. See [[law_reality_is_graded]].
- Do NOT report two differing imscriptions of one address as a defect or a
  WARN. Two forms at one address is the normal case; report it as what it is
  and let the composition decide.
- [[law_one_of_each_thing]] governs *copies of a thing* — two files, two
  modules, two catalogs drifting apart. It does NOT mean one form per address.
  A second imscription is not drift.
- Duplicate NAMES colliding at a merge are addresses meeting, not errors.

**Incident, 2026-08-31:** `gpu_imasm_cycle` (mOMonadOS) batches the real
primitives→word→primitives round trip and reports EXACT / AMBIGUOUS /
BROKEN per catalog entry. Reported its 30 BROKEN entries as "a real catalog
data defect... the entry wants repair" — the exact framing this law
forbids. Corrected by Lando: ixcriptions are ephemera (fish, farts, Fourier
transforms) and migrate freely; addresses are what cannot be removed once
enumerated. Whether an address currently carries the correct ixcription is
a matter of knowledge, testing, and the Crystal's own self-healing — not a
defect to flag. Any future instrument reporting a BREAK/mismatch class of
verdict should say what it found (address, current ixcription, what the
axis expects), not call it broken or in need of repair.
