#!/usr/bin/env python3
"""Discover, build and operate the local programs without writing driver code."""
import argparse
from datetime import datetime
import json
import os
from pathlib import Path
import re
import shlex
import shutil
import subprocess
import sys
import tomllib
import uuid

VOX = Path(__file__).resolve().parents[1]
WORKSPACE = VOX.parent
sys.path.insert(0, str(VOX))
from standalone import require_standalone, static_env

FAMILIES = {
    "factor": "Vox source-only factor membrane",
    "resident": "Cargo-prepared resident register with native Y and clock",
    "rust-phase": "Rust phase membrane",
    "c-phase": "C phase membrane plus IMASM and glyph sidecars",
    "ququart": "Prepared ququart membrane and independent verifier",
    "gpu": "Prepared GPU idempotent membrane with embedded device code",
    "cpu": "CPU membrane from an existing prepared GPU source directory",
    "word": "Native executable from a saved IMASM word",
}


class CommandFailure(RuntimeError):
    def __init__(self, command, code):
        self.code = code if code >= 0 else 128 - code
        super().__init__(f"Command exited {code}: {shlex.join(map(str, command))}")


def description(path):
    if not path.is_file():
        return "Source file is unavailable"
    with path.open(errors="replace") as source:
        for _, line in zip(range(24), source):
            line = line.strip()
            if line.startswith(("//!", "# ")) and not line.startswith("#!/"):
                return line.lstrip("/!# ")[:110]
    return path.name


def crate_catalog(repo, prefix=""):
    manifest = tomllib.loads((repo / "Cargo.toml").read_text())
    entries = {}
    for row in manifest.get("bin", []):
        path = repo / row.get("path", f"src/bin/{row['name']}.rs")
        entries[f"{prefix}bin:{row['name']}"] = ("bin", row["name"], path)
    if manifest.get("package", {}).get("autobins", True):
        declared = {v[2] for v in entries.values()}
        automatic = [(p.stem, p) for p in (repo / "src/bin").glob("*.rs")]
        automatic += [(p.parent.name, p) for p in (repo / "src/bin").glob("*/main.rs")]
        if (repo / "src/main.rs").is_file():
            automatic.append((manifest["package"]["name"], repo / "src/main.rs"))
        for name, path in automatic:
            if path not in declared:
                entries.setdefault(f"{prefix}bin:{name}", ("bin", name, path))
    examples = {p.stem: p for p in (repo / "examples").glob("*.rs")}
    for row in manifest.get("example", []):
        examples[row["name"]] = repo / row.get("path", f"examples/{row['name']}.rs")
    for name, path in examples.items():
        entries[f"{prefix}example:{name}"] = ("example", name, path)
    return entries


def catalog(repo):
    entries = crate_catalog(repo)
    # These are live nested projects. Vendor and proof snapshots retain their
    # own historical build contracts and are not alternate copies to operate.
    for directory in (repo / "membranes", repo / "gpu_port"):
        if directory.is_dir():
            for current, directories, files in os.walk(directory):
                directories[:] = [name for name in directories
                                  if name not in ("target", ".git", "vendor")
                                  and not (Path(current) / name).is_symlink()]
                if "Cargo.toml" in files:
                    crate = Path(current)
                    prefix = str(crate.relative_to(repo)) + "/"
                    entries.update(crate_catalog(crate, prefix))
    scripts = set()
    for directory in (repo, repo / "tools", repo / "scripts", repo / "py"):
        if directory.is_dir():
            scripts.update(directory.glob("*.sh"))
            scripts.update(directory.glob("*.py"))
    inventory = WORKSPACE / "ig-docs/membrane_bakers_static_20261015.json"
    if inventory.is_file():
        for row in json.loads(inventory.read_text()).get("bakers", []):
            path = WORKSPACE / row["path"]
            if path.is_relative_to(repo) and path.suffix in (".sh", ".py"):
                scripts.add(path)
    for path in scripts:
        if path.name == "programs.py":
            continue
        name = str(path.relative_to(repo))
        entries[f"script:{name}"] = ("script", name, path)
    return entries


def resolve(repo, name):
    entries = catalog(repo)
    if name in entries:
        return entries[name]
    matches = [v for v in entries.values() if v[1] == name]
    if len(matches) == 1:
        return matches[0]
    if len(matches) > 1:
        raise ValueError(f"Ambiguous name: {name}. Use bin:, example: or script:.")
    path = (repo / name).resolve()
    if path.is_relative_to(repo) and path.is_file() and path.suffix in (".py", ".sh"):
        return "script", str(path.relative_to(repo)), path
    raise ValueError(f"Unknown program: {name}. Find it with ./programs list --all {shlex.quote(name)}")


def invoke(command, cwd, env=None, capture=False):
    result = subprocess.run([str(v) for v in command], cwd=cwd, env=env,
                            text=True, stdout=subprocess.PIPE if capture else None)
    if result.returncode:
        raise CommandFailure(command, result.returncode)
    return result.stdout if capture else None


def binary(repo, name, args, kind="bin"):
    target = "x86_64-unknown-linux-musl" if repo == VOX else "x86_64-unknown-linux-gnu"
    profile = "debug" if args.debug else "release"
    path = repo / "target" / target / profile
    if kind == "example":
        path /= "examples"
    path /= name
    if not args.no_build:
        command = ["cargo", "build", "--manifest-path", str(repo / "Cargo.toml"),
                   "--target", target, "--target-dir", str(repo / "target"), f"--{kind}", name]
        if not args.debug:
            command.append("--release")
        manifest = tomllib.loads((repo / "Cargo.toml").read_text())
        entry = next((row for row in manifest.get(kind, []) if row["name"] == name), {})
        if entry.get("required-features"):
            command += ["--features", ",".join(entry["required-features"])]
        invoke(command, repo, static_env())
    if not path.is_file():
        raise ValueError(f"Executable is missing: {path}. Run again without --no-build.")
    require_standalone(path)
    return path


def show_help(repo, name):
    requested = name
    kind, name, path = resolve(repo, name)
    print(f"{name}: {description(path)}\nSource: {path}")
    print(f"Run: ./programs run {shlex.quote(requested)} [arguments]")
    if kind in ("bin", "example"):
        print(f"Build only: ./programs build {shlex.quote(requested)}")
    if not path.is_file():
        raise ValueError("Restore the source file before building this program.")
    lines = path.read_text(errors="replace").splitlines()
    usage = []
    for line in lines:
        if re.search(r"usage:|Usage:|add_argument\(|ArgumentParser\(|^#   [A-Za-z.<]", line):
            literal = re.search(r'(?:e?println!|format!)\(\s*"((?:[^"\\]|\\.)*)"', line)
            if literal:
                text = literal[1].replace('\\n', '\n').replace('\\t', '\t').replace('\\"', '"')
            else:
                text = line.strip().removeprefix('//!').removeprefix('#').strip()
            usage.append(text)
    print("\nUsage recorded in its source:")
    print("\n".join(usage[:24]) if usage else
          "No usage text is recorded. Use the main CLI help or the recipe in USER_GUIDE.md.")
    if '"--help"' in "\n".join(lines) or "'--help'" in "\n".join(lines):
        print(f"Live help: ./programs run {shlex.quote(requested)} --help")
    if "option_env!(" in "\n".join(lines) or 'env!("' in "\n".join(lines):
        print("\nThis source has build-time settings. For instance-bound membranes, use ./programs bake.")
    print("\nHelp inspection does not start the program. Relative program arguments use this repository as their working directory.")


def execute_image(path, args):
    path = path.expanduser().resolve()
    if not path.is_file():
        raise ValueError(f"File does not exist: {path}")
    if path.suffix in (".imasm", ".glyphs"):
        command = [binary(VOX, "vox", args), "run", path]
    else:
        require_standalone(path)
        command = [path]
    invoke(command, Path.cwd())


def bake(args):
    if args.list:
        for name, text in FAMILIES.items():
            print(f"{name:12} {text}")
        return
    family = args.family or ("factor" if args.repo == VOX else "resident")
    gm = WORKSPACE / "G-mOMonadOS"
    home = VOX if family == "factor" else gm
    output = args.output.expanduser().resolve() if args.output else home / "membranes" / (
        f"{family}_{datetime.now():%Y%m%d_%H%M%S}_{uuid.uuid4().hex[:6]}")
    if output.exists():
        raise ValueError(f"Output already exists: {output}. Choose a new --output path.")
    # Each baker applies the shared static policy itself. Preserve its exact
    # flags here so repeated launches reuse the same Cargo artifacts.
    env = dict(os.environ)
    env.pop("MEMBRANE_RUN", None)
    output.parent.mkdir(parents=True, exist_ok=True)
    number = args.number
    if args.input:
        number = args.input.expanduser().resolve().read_text().strip()
    if family not in ("cpu", "word") and not args.word_file:
        if number is None and sys.stdin.isatty():
            number = input("Decimal source N: ").strip()
        if not number or not re.fullmatch(r"[1-9][0-9]*", number) or int(number) <= 1:
            raise ValueError("Supply --number DECIMAL_N or --input FILE containing N alone.")
    if args.word_file and family not in ("ququart", "word"):
        raise ValueError("--word-file is supported by ququart and word bakers.")
    if family == "factor":
        invoke(["bash", VOX / "bake_factor.sh", number, output], VOX, env)
        image = output / "factor_one"
    elif family == "resident":
        invoke(["bash", gm / "bake_resident.sh", number, output], gm, env)
        image = output / "resident_factor_baked"
    elif family in ("rust-phase", "c-phase"):
        script = "imasm_phase_factor_build.sh" if family == "rust-phase" else "vox_phase_factor_build.sh"
        if family == "c-phase":
            env["VOX_BIN"] = str(binary(VOX, "vox", args))
        invoke(["bash", gm / script, number, str(args.base), output], gm, env)
        image = output
    elif family == "gpu":
        invoke([binary(gm, "g-momonados", args), "idempotent", "emit", number, output], gm, env)
        image = output
    elif family == "cpu":
        if not args.prepared:
            raise ValueError("The CPU baker needs --prepared PREPARED_SOURCE_DIRECTORY.")
        invoke(["bash", gm / "tools/bake_idempotent_cpu.sh", args.prepared.resolve(), output], gm, env)
        image = output
    elif family == "word":
        if not args.word_file:
            raise ValueError("The word baker needs --word-file FILE.imasm.")
        invoke([sys.executable, gm / "imasm_emit.py", args.word_file.resolve(), str(args.line), output], gm, env)
        image = output
    else:
        codec = binary(VOX, "vox", args)
        if args.word_file:
            source = "@" + str(args.word_file.resolve())
        else:
            source = invoke([codec, "numeral", number], VOX, capture=True).strip()
        base = invoke([codec, "numeral", str(args.base)], VOX, capture=True).strip()
        command = [sys.executable, gm / "prepare_ququart.py", source, output, "--base", base]
        if args.native_fourier:
            command.append("--native-fourier")
        if args.retained_case:
            command += ["--retained-case", args.retained_case.resolve()]
        invoke(command, gm, env)
        image = output / "membrane"
    require_standalone(image)
    image = image.resolve()
    print(f"\nReady: {image}\nRun: {shlex.quote(str(image))}")


def parser():
    p = argparse.ArgumentParser(prog="./programs", description=__doc__, epilog="Start with list, help PROGRAM, bake --help, or menu. No program runs when no command is supplied.")
    p.add_argument("--repo", type=Path, default=VOX, help=argparse.SUPPRESS)
    p.add_argument("--debug", action="store_true", help="use a debug build instead of release")
    p.add_argument("--no-build", action="store_true", help="reuse existing Rust tools; bake recipes still compile their outputs")
    sub = p.add_subparsers(dest="command")
    ls = sub.add_parser("list", help="search programs without building or running them")
    ls.add_argument("query", nargs="?", default="")
    ls.add_argument("--all", action="store_true", help="include examples, shell and Python programs")
    ls.add_argument("--json", action="store_true")
    h = sub.add_parser("help", help="read a program's usage without executing it")
    h.add_argument("program")
    for name in ("run", "build"):
        command = sub.add_parser(name, help=f"{name} one named program")
        command.add_argument("program")
        if name == "run":
            command.add_argument("arguments", nargs=argparse.REMAINDER)
    cli = sub.add_parser("cli", help="open the main CLI or pass it arguments")
    cli.add_argument("arguments", nargs=argparse.REMAINDER)
    ex = sub.add_parser("execute", help="run a saved static membrane, IMASM or glyph file")
    ex.add_argument("file", type=Path)
    b = sub.add_parser("bake", help="bake a static membrane from ordinary inputs")
    b.add_argument("family", choices=FAMILIES, nargs="?")
    b.add_argument("--list", action="store_true")
    source = b.add_mutually_exclusive_group()
    source.add_argument("--number", help="decimal source N")
    source.add_argument("--input", type=Path, help="file containing decimal N alone")
    source.add_argument("--word-file", type=Path, help="saved IMASM source for ququart or word")
    b.add_argument("--output", type=Path, help="fresh output path, relative to your current directory")
    b.add_argument("--base", type=int, default=2, help="decimal phase base (default: 2)")
    b.add_argument("--prepared", type=Path, help="prepared source directory for the CPU baker")
    b.add_argument("--line", type=int, default=1, help="nonempty source line for word baking (default: 1)")
    modes = b.add_mutually_exclusive_group()
    modes.add_argument("--native-fourier", action="store_true", help="use the ququart baker's native Fourier preparation")
    modes.add_argument("--retained-case", type=Path, help="reuse a compatible retained ququart preparation")
    sub.add_parser("doctor", help="check local build tools and target availability")
    sub.add_parser("menu", help="choose a program and enter its arguments interactively")
    return p


def main(argv=None):
    p = parser()
    args = p.parse_args(argv)
    args.repo = args.repo.resolve()
    if args.repo not in (VOX, WORKSPACE / "G-mOMonadOS"):
        p.error("Select the G-mOMonadOS or Vox directory.")
    try:
        if not args.command:
            p.print_help()
        elif args.command == "list":
            rows = [{"name": key, "kind": v[0], "source": str(v[2]),
                     "available": v[2].is_file(), "description": description(v[2])}
                    for key, v in sorted(catalog(args.repo).items())
                    if (args.all or v[0] == "bin")
                    and args.query.lower() in (key + " " + description(v[2])).lower()]
            if args.json:
                print(json.dumps(rows, indent=2))
            else:
                for row in rows:
                    print(f"{row['name']:45} {row['description']}")
                if not rows:
                    print("No matching programs. Try ./programs list --all.")
        elif args.command == "help":
            show_help(args.repo, args.program)
        elif args.command == "doctor":
            missing = False
            for tool in ("python3", "cargo", "rustup", "bash", "cc", "g++", "readelf", "node"):
                path = shutil.which(tool)
                print(f"{tool:12} {path or 'missing'}")
                missing |= path is None and tool in ("cargo", "bash")
            if shutil.which("rustup"):
                invoke(["rustup", "target", "list", "--installed"], args.repo)
            print("Rust bakers need their Linux target. C/CPU bakers also need a static C/C++ runtime.\nGPU execution needs a compatible CUDA driver and device. No computation was started.")
            return int(missing)
        elif args.command == "bake":
            if args.base < 2 or args.line < 1:
                raise ValueError("--base must be at least 2; --line must be positive.")
            bake(args)
        elif args.command == "execute":
            execute_image(args.file, args)
        elif args.command == "menu":
            print("Enter a program name from the list. Type q to leave.\n")
            entries = catalog(args.repo)
            for key in sorted(entries):
                if entries[key][0] == "bin":
                    print(key)
            while True:
                name = input("\nProgram (or q): ").strip()
                if name in ("q", "quit", "exit"):
                    return 0
                try:
                    show_help(args.repo, name)
                    arguments = shlex.split(input("Arguments (shell quoting accepted, no shell expansion): "))
                    return main(["--repo", str(args.repo), *( ["--no-build"] if args.no_build else []),
                                 *( ["--debug"] if args.debug else []), "run", name, *arguments])
                except ValueError as error:
                    print(error, file=sys.stderr)
        else:
            name = ("vox" if args.repo == VOX else "g-momonados") if args.command == "cli" else args.program
            kind, name, path = resolve(args.repo, name)
            if kind == "script":
                if args.command == "build":
                    raise ValueError("This is a script. Use run, or bake to produce a membrane.")
                command = [sys.executable if path.suffix == ".py" else "bash", path]
            else:
                crate = next(parent for parent in path.parents if (parent / "Cargo.toml").is_file())
                command = [binary(crate, name, args, kind)]
            if args.command == "build":
                print(command[0])
            else:
                extra = args.arguments
                invoke([*command, *(extra[1:] if extra[:1] == ["--"] else extra)], args.repo)
        return 0
    except KeyboardInterrupt:
        print("\nStopped by request.", file=sys.stderr)
        return 130
    except EOFError:
        print("Input ended. Supply arguments directly or run menu in a terminal.", file=sys.stderr)
        return 2
    except CommandFailure as error:
        print(f"programs: {error}", file=sys.stderr)
        return error.code
    except (OSError, ValueError, RuntimeError) as error:
        print(f"programs: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    sys.exit(main())
