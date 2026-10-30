#!/usr/bin/env python3
"""Discover, build and operate the local programs without writing driver code."""
import argparse
from datetime import datetime
import json
import os
from pathlib import Path
import re
import signal
import shlex
import shutil
import subprocess
import sys
import tomllib
import uuid

VOX = Path(__file__).resolve().parents[1]
WORKSPACE = VOX.parent
G = WORKSPACE / "G-mOMonadOS"
M3IOSIS = WORKSPACE / "m3iosis"
M3IOSIS_CRATE = M3IOSIS / "src/m3iosis"
M3MBR4N3S = WORKSPACE / "m3mbr4n3s"
M3MBR4N3S_CRATE = M3MBR4N3S / "membranes"
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
    if repo == M3IOSIS:
        entries = crate_catalog(M3IOSIS_CRATE, "src/m3iosis/")
    elif repo == M3MBR4N3S:
        entries = crate_catalog(M3MBR4N3S_CRATE, "membranes/")
    else:
        entries = crate_catalog(repo)
    # These are live nested projects. Vendor and proof snapshots retain their
    # own historical build contracts and are not alternate copies to operate.
    nested_projects = [repo / "membranes", repo / "gpu_port"]
    if repo == M3IOSIS:
        nested_projects.append(M3IOSIS_CRATE / "membranes")
    for directory in nested_projects:
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
    script_directories = [repo, repo / "tools", repo / "scripts", repo / "py"]
    if repo == M3IOSIS:
        script_directories.extend((M3IOSIS_CRATE, M3IOSIS_CRATE / "tools", M3IOSIS_CRATE / "scripts"))
    elif repo == M3MBR4N3S:
        script_directories.extend((M3MBR4N3S_CRATE, M3MBR4N3S_CRATE / "tools", M3MBR4N3S_CRATE / "scripts"))
    for directory in script_directories:
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


def membrane_catalog(repo):
    selected = {}
    support_entries = {
        "double_arev_membrane", "single_frame_membrane", "glyph_membranes.sh",
        "membrane_dissolve.py", "membrane_run_15.sh", "membrane_run_big.sh",
        "static_membrane.sh", "imasm_emit.py",
    }
    for key, (kind, name, path) in catalog(repo).items():
        if path.name in support_entries or path.stem in support_entries:
            continue
        relative = str(path.relative_to(repo)) if path.is_relative_to(repo) else str(path)
        if kind == "bin":
            package_target = "/membranes/" in f"/{relative}/"
            mentions_membrane = path.is_file() and re.search(
                r"\bmembrane\b", path.read_text(errors="replace"), re.IGNORECASE
            )
            if package_target or "membrane" in name.lower() or mentions_membrane:
                selected[key] = (kind, name, path)
        elif kind == "script" and path.suffix in (".py", ".sh"):
            source = path.read_text(errors="replace") if path.is_file() else ""
            filename = path.name.lower()
            if (
                "membrane" in filename
                or filename == "baked_case.py"
                or (filename == "imasm_emit.py" and repo == G)
                or (path.suffix == ".py" and re.search(
                    r"class\s+\w*Membrane|def\s+build_case\s*\(", source
                ))
            ):
                selected[key] = (kind, name, path)
    return selected


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


def baked_number_contract(path):
    if not path.is_file() or path.suffix != ".rs":
        return None
    source = path.read_text(errors="replace")
    if path.stem == "resident_factor_baked":
        return "RESIDENT_SOURCE_FILE"
    if "GLUT_SOURCE_WORD" in source:
        return "GLUT_SOURCE_WORD"
    for variable in ("MEMBRANE_WORDS", "MEMBRANE_DECIMALS", "FACTOR_N_WORD", "PERFECT_N_WORD",
                     "FRAME_FACTOR_N_WORD", "GODEL_SOURCE_WORD", "FACTOR_N_DEC"):
        if f'option_env!("{variable}")' in source or f'match option_env!("{variable}")' in source:
            return variable
    if "RESIDENT_SOURCE_FILE" in source or "BAKED_SOURCE_WORD" in source:
        return "RESIDENT_SOURCE_FILE"
    if "VOX_BAKED_INPUT_FILE" in source or "BAKED_MODULUS_WORD" in source:
        return "VOX_BAKED_INPUT_FILE"
    return None


def baked_number_environment(repo, name, path, number, args):
    contract = baked_number_contract(path)
    if not contract:
        raise ValueError(f"{name} does not declare a build-time decimal N input.")
    numbers = number if isinstance(number, list) else [number]
    if not numbers or any(not re.fullmatch(r"[1-9][0-9]*", value) or int(value) <= 1 for value in numbers):
        raise ValueError("Each --number must be a decimal integer greater than 1.")
    env = static_env()
    if contract in ("RESIDENT_SOURCE_FILE", "VOX_BAKED_INPUT_FILE"):
        encoder = binary(VOX, "vox", args)
        words = [invoke([encoder, "numeral", value], VOX, capture=True).strip() for value in numbers]
        baked_inputs = repo / "target" / "membrane-inputs"
        baked_inputs.mkdir(parents=True, exist_ok=True)
        source_file = baked_inputs / f"{name}.imasm"
        source_file.write_text(" ".join(words) + "\n")
        env[contract] = str(source_file)
    elif contract == "MEMBRANE_DECIMALS":
        env[contract] = " ".join(numbers)
    elif contract.endswith("_DEC"):
        if len(numbers) != 1:
            raise ValueError(f"{name} accepts exactly one --number.")
        env[contract] = numbers[0]
    else:
        encoder = binary(VOX, "vox", args)
        words = [invoke([encoder, "numeral", value], VOX, capture=True).strip() for value in numbers]
        env[contract] = " ".join(words)
    return env


def source_graph(path):
    crate = next((parent for parent in path.parents if (parent / "Cargo.toml").is_file()), None)
    pending = [path]
    visited = set()
    while pending:
        current = pending.pop().resolve()
        if current in visited or not current.is_file() or current.suffix not in (".rs", ".py", ".sh"):
            continue
        visited.add(current)
        text = current.read_text(errors="replace")
        for relative in re.findall(r'#\[path\s*=\s*"([^"]+)"\]', text):
            pending.append(current.parent / relative)
        if current.suffix == ".rs":
            for module in re.findall(r'(?m)^\s*mod\s+([A-Za-z_]\w*)\s*;', text):
                options = (current.parent / f"{module}.rs", current.parent / module / "mod.rs")
                pending.extend(option for option in options if option.is_file())
            if crate:
                manifest = tomllib.loads((crate / "Cargo.toml").read_text())
                aliases = {manifest.get("package", {}).get("name", "").replace("-", "_")}
                aliases.add(manifest.get("lib", {}).get("name", ""))
                aliases.update(name.replace("-", "_") for name, dependency in manifest.get("dependencies", {}).items()
                               if isinstance(dependency, dict) and "path" in dependency)
                for alias in filter(None, aliases):
                    for module in re.findall(rf'\b{re.escape(alias)}::([A-Za-z_]\w*)::', text):
                        candidates = (crate / "src" / f"{module}.rs", crate / "src" / module / "mod.rs")
                        pending.extend(candidate for candidate in candidates if candidate.is_file())
    return [Path(value) for value in visited]


def cli_flags(path):
    found = set()
    usage = []
    for source in source_graph(path):
        text = source.read_text(errors="replace")
        if re.search(r"membranes::main_membrane\s*\(", text):
            found.update(("--input", "--steps"))
        if source.suffix == ".py":
            for argument in re.findall(r'add_argument\s*\((.*?)\)', text, re.DOTALL):
                found.update(re.findall(r'["\'](--[A-Za-z][A-Za-z0-9-]*)["\']', argument))
                found.update(re.findall(r'["\'](-[A-Za-z])["\']', argument))
            if "ArgumentParser(" in text:
                found.add("--help")
            usage.extend(line.strip().lstrip("# ") for line in text.splitlines()
                         if re.search(r'usage:', line, re.IGNORECASE))
            continue
        for line in text.splitlines():
            if re.search(r'usage|Usage|args|argument|add_argument|\.arg\(|\.long\(|strip_prefix|matches!|--[a-zA-Z]', line):
                if not re.search(r'\bcargo\s+(?:run|build)\b', line):
                    found.update(re.findall(r'["\'](--[A-Za-z][A-Za-z0-9-]*)["\']', line))
                    found.update(re.findall(r'["\'](-[A-Za-z])["\']', line))
                    found.update(re.findall(r'(?<![\w-])(--[A-Za-z][A-Za-z0-9-]*)', line))
                usage_match = re.search(r'usage:\s*([^"\']+)', line, re.IGNORECASE)
                if usage_match:
                    usage.append(usage_match[1].strip().rstrip(');'))
                for signature in re.findall(r'"([^"]*(?:factor-membrane N|factor-membrane <N>)[^"]*)"', line):
                    usage.append(signature.split(r"\n", 1)[0])
    return sorted(found), usage


def display_flags(path):
    if path.name == "baked_case.py":
        return ["--mode MODE", "--number DECIMAL_N", "--input FILE", "--width QUBITS", "--radix RADIX", "--output FILE"]
    if baked_number_contract(path):
        values = ["--number DECIMAL_N (build-time)"]
        if baked_number_contract(path) in ("MEMBRANE_WORDS", "MEMBRANE_DECIMALS"):
            values[0] += " (repeat for each decimal input)"
        values.append("--run (execute after baking)")
        return values
    if path.is_file() and "membranes::main_membrane(" in path.read_text(errors="replace"):
        return ["--input CANONICAL_IMASM_WORD", "--steps COUNT"]
    values = {
        "--b1": "BOUND", "--b2": "BOUND", "--base": "DECIMAL_BASE", "--bits": "BITS...",
        "--curves": "COUNT", "--family": "FAMILY", "--first-sigma": "SIGMA",
        "--factors": "P Q", "--hex-word": "HEX_WORD", "--input": "FILE",
        "--leap-cap": "COUNT", "--node-cap": "COUNT",
        "--inspect": "WORD DECIMAL_SOURCE", "--line": "NUMBER", "--mode": "MODE",
        "--native-word": "WORD", "--number": "DECIMAL_N", "--output": "FILE",
        "--passes": "COUNT", "--qubits": "COUNT", "--radix": "RADIX", "--shots": "COUNT", "--steps": "COUNT", "--strategy": "STRATEGY",
        "--target": "TRIPLE", "--target-dir": "DIR", "--verify": "FACTOR_WORD...",
        "--width": "QUBITS",
        "-e": "COUNT", "-f": "COUNT", "-k": "COUNT", "-o": "ORDER", "-s": "STRATEGY", "-w": "WORD",
    }
    flags, _ = cli_flags(path)
    if "G-mOMonadOS" in str(path):
        values["--number"] = "CANONICAL_IMASM_N"
    return [f"{flag} {values[flag]}" if flag in values else flag for flag in flags]


def cli_example(key, path):
    if path.name == "baked_case.py":
        return "./programs membranes bake --mode divisor --number DECIMAL_N --output FILE"
    if baked_number_contract(path):
        if baked_number_contract(path) in ("MEMBRANE_WORDS", "MEMBRANE_DECIMALS"):
            return f"./programs membranes build {key} --number DECIMAL_N --number DECIMAL_M [--run]"
        return f"./programs membranes build {key} --number DECIMAL_N [--run]"
    if path.is_file() and "membranes::main_membrane(" in path.read_text(errors="replace"):
        return f"./programs membranes run {key} --input CANONICAL_IMASM_WORD --steps COUNT"
    if path.name == "membrane_one.sh":
        return f"./programs membranes run {key} --number DECIMAL_N"
    if path.name == "multi_membrane_one.sh":
        return f"./programs membranes run {key} --number DECIMAL_N --base DECIMAL_A --qubits COUNT"
    if path.name == "vox_membrane.sh":
        return f"./programs membranes run {key} --input FILE.imasm --output FILE"
    if path.name == "make_membrane.sh":
        return f"./programs membranes run {key} --number DECIMAL_N --base DECIMAL_A --shots COUNT"
    flags, usage = cli_flags(path)
    if "factor_membrane" in key and "--dialectic-product" in flags:
        return f"./programs membranes run {key} DECIMAL_N --dialectic-product"
    if "--inspect" in flags:
        return f"./programs membranes run {key} --inspect IMASM_WORD DECIMAL_SOURCE"
    if "--help" in flags:
        separator = " -- " if key.startswith("script:") else " "
        return f"./programs membranes run {key}{separator}--help"
    if flags:
        values = {
            "--base": "DECIMAL_A", "--b1": "BOUND", "--b2": "BOUND",
            "--curves": "COUNT", "--first-sigma": "SIGMA", "--input": "CANONICAL_IMASM_WORD",
            "--leap-cap": "COUNT", "--node-cap": "COUNT", "--number": "DECIMAL_N",
            "--qubits": "COUNT", "--steps": "COUNT",
        }
        if "G-mOMonadOS" in str(path):
            values["--number"] = "CANONICAL_IMASM_N"
        options = [f"{flag} {values[flag]}" if flag in values else flag for flag in flags]
        return f"./programs membranes run {key} " + " ".join(options)
    if usage:
        signature = usage[0]
        signature = re.sub(r'^Usage:\s*', '', signature, flags=re.IGNORECASE)
        signature = re.sub(r'^cargo\s+run\s+--bin\s+\S+\s*', '', signature)
        entry_name = re.escape(key.rsplit(":", 1)[-1])
        signature = re.sub(rf'^{entry_name}\s*', '', signature)
        signature = re.sub(r'\s+\[.*$', '', signature)
        signature = re.sub(r'<([^>]+)>', lambda match: match[1].replace("-", "_").upper(), signature)
        signature = re.sub(r'\s+\([^)]*\).*$', '', signature)
        if signature:
            return f"./programs membranes run {key} {signature}"
    return f"./programs membranes run {key}"


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
        source = repo / "src/bin" / f"{name}.rs"
        contract = baked_number_contract(source)
        if contract and not getattr(args, "number", None):
            if sys.stdin.isatty():
                args.number = input(f"Decimal N to bake into {name}: ").strip()
            else:
                raise ValueError(f"{name} needs a baked decimal input. Supply --number DECIMAL_N.")
        if contract and getattr(args, "number", None):
            env = baked_number_environment(repo, name, source, args.number, args)
        else:
            env = static_env()
        invoke(command, repo, env)
    elif baked_number_contract(repo / "src/bin" / f"{name}.rs") and getattr(args, "number", None):
        raise ValueError("--no-build cannot bake a new N; remove --no-build to compile the requested input.")
    if not path.is_file():
        raise ValueError(f"Executable is missing: {path}. Run again without --no-build.")
    require_standalone(path)
    return path


def show_help(repo, name):
    requested = name
    kind, name, path = resolve(repo, name)
    print(f"{name}: {description(path)}\nSource: {path}")
    contract = baked_number_contract(path)
    if contract:
        print(f"Build with input: ./programs membranes build {shlex.quote(requested)} --number DECIMAL_N")
        print(f"Run after baking: ./programs membranes run {shlex.quote(requested)}")
        print(f"Bake and run:     ./programs membranes build {shlex.quote(requested)} --number DECIMAL_N --run")
    else:
        print(f"Run example: {cli_example(requested, path)}")
    flags, usage = cli_flags(path)
    shown_flags = display_flags(path)
    print("CLI flags: " + (", ".join(shown_flags) if shown_flags else "No named CLI flags detected"))
    if usage:
        print("Usage signatures:\n" + "\n".join(usage[:12]))
    if kind in ("bin", "example"):
        if contract:
            print(f"Build only: ./programs build {shlex.quote(requested)} --number DECIMAL_N")
        else:
            print(f"Build only: ./programs build {shlex.quote(requested)}")
    if not path.is_file():
        raise ValueError("Restore the source file before building this program.")
    lines = path.read_text(errors="replace").splitlines()
    source_notes = []
    for line in lines:
        if line.startswith(("//!", "///")):
            source_notes.append(line.lstrip("/! ").strip())
        elif source_notes:
            break
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
    if source_notes:
        print("\nEntry notes:\n" + "\n".join(source_notes))
    if '"--help"' in "\n".join(lines) or "'--help'" in "\n".join(lines):
        print(f"Live help: ./programs run {shlex.quote(requested)} --help")
    if "option_env!(" in "\n".join(lines) or 'env!("' in "\n".join(lines):
        print("\nBuild-time settings are baked into the emitted executable.")
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


def render_menu(entries, repo_name="Vox", query="", page=0):
    rows = [(key, value) for key, value in sorted(entries.items())
            if value[0] == "bin" and query.lower() in (key + " " + description(value[2])).lower()]
    page_size = 5
    page_count = max(1, (len(rows) + page_size - 1) // page_size)
    page = min(page, page_count - 1)
    visible = rows[page * page_size:(page + 1) * page_size]
    try:
        from rich.console import Console
        from rich.panel import Panel
        from rich.table import Table
        from rich.text import Text
        console = Console()
        table = Table(box=None, expand=True, header_style="bold cyan", show_lines=False)
        table.add_column("#", style="bold yellow", justify="right", width=4)
        table.add_column("Entry · flags · example", overflow="fold")
        for index, (key, (_, _, path)) in enumerate(visible, page * page_size + 1):
            flags = display_flags(path)
            details = Text(key, style="bold green")
            details.append("\nFlags: " + (", ".join(flags) if flags else "No CLI flags detected for this entry"), style="bright_magenta")
            details.append("\nExample: " + cli_example(key, path), style="yellow")
            details.append("\n" + description(path), style="white")
            table.add_row(str(index), details)
        title = f"{repo_name} Program Menu" if not query else f"{repo_name} Program Menu · {query}"
        console.print(Panel(table, title=f"[bold bright_cyan]{title}[/]",
                            subtitle=f"[dim]Page {page + 1}/{page_count} · n/p page · /text filter · q quit[/]",
                            border_style="cyan", padding=(1, 2)))
    except ImportError:
        print(f"\n{repo_name.upper()} PROGRAM MENU" + (f" · {query}" if query else ""))
        print(f"Page {page + 1}/{page_count} · n/p page · /text filter · q quit")
        print("─" * 78)
        for index, (key, (_, _, path)) in enumerate(visible, page * page_size + 1):
            flags = display_flags(path)
            print(f"{index:>3}  {key}\n     Flags: {', '.join(flags) if flags else 'No CLI flags detected for this entry'}")
            print(f"     Example: {cli_example(key, path)}\n     {description(path)}")
        print("─" * 78)
        print("Enter a number or name, /text to filter, or q to quit.")
    return rows


def bake(args):
    if args.list:
        for name, text in FAMILIES.items():
            print(f"{name:12} {text}")
        return
    if getattr(args, "mode", None):
        if not args.number or not re.fullmatch(r"[1-9][0-9]*", args.number) or int(args.number) <= 1:
            raise ValueError("Supply --number DECIMAL_N greater than 1 for mode baking.")
        if args.width is not None and args.width < 1:
            raise ValueError("--width must be positive.")
        import baked_case
        if args.mode not in baked_case.MODES:
            raise ValueError(f"Unknown mode {args.mode}; choose from: {', '.join(baked_case.MODES)}")
        baked_case.TARGET_DIR = args.repo / "target"
        image, _ = baked_case.build_case(int(args.number), args.mode, args.width, radix=args.radix)
        image = image.resolve()
        if args.output:
            destination = args.output.expanduser().resolve()
            if destination.exists():
                raise ValueError(f"Output already exists: {destination}")
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(image, destination)
            image = destination
        require_standalone(image)
        print(f"\nReady: {image}\nRun: {shlex.quote(str(image))}")
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
        command.add_argument("--number", action="append", help="decimal input baked into compile-time-input programs; repeat for multiple inputs")
        if name == "build":
            command.add_argument("--run", action="store_true", help="run the emitted binary after building")
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
    b.add_argument("--mode", help="baked_case per-instance mode")
    b.add_argument("--list-modes", action="store_true", help="list per-instance baked_case modes")
    b.add_argument("--width", type=int, help="mode-specific width")
    b.add_argument("--radix", type=int, help="divisor mode prefix-closure radix")
    b.add_argument("--base", type=int, default=2, help="decimal phase base (default: 2)")
    b.add_argument("--prepared", type=Path, help="prepared source directory for the CPU baker")
    b.add_argument("--line", type=int, default=1, help="nonempty source line for word baking (default: 1)")
    modes = b.add_mutually_exclusive_group()
    modes.add_argument("--native-fourier", action="store_true", help="use the ququart baker's native Fourier preparation")
    modes.add_argument("--retained-case", type=Path, help="reuse a compatible retained ququart preparation")
    membranes = sub.add_parser("membranes", help="list, build, run or inspect membrane programs")
    membrane_actions = membranes.add_subparsers(dest="membrane_action")
    membrane_list = membrane_actions.add_parser("list", help="list membrane binaries and Python bakers")
    membrane_list.add_argument("query", nargs="?", default="")
    membrane_list.add_argument("--examples", action="store_true", help="show a command example for every entry")
    membrane_list.add_argument("--json", action="store_true")
    membrane_help = membrane_actions.add_parser("help", help="show source usage and options")
    membrane_help.add_argument("program")
    membrane_build = membrane_actions.add_parser("build", help="emit a standalone Rust membrane binary")
    membrane_build.add_argument("program")
    membrane_build.add_argument("--number", action="append", help="decimal input baked into compile-time-input programs; repeat for multiple inputs")
    membrane_build.add_argument("--run", action="store_true", help="run the emitted binary after baking")
    membrane_run = membrane_actions.add_parser("run", help="run an entry until completion or Ctrl+C")
    membrane_run.add_argument("program")
    membrane_run.add_argument("arguments", nargs=argparse.REMAINDER)
    membrane_bake = membrane_actions.add_parser("bake", help="emit a source-bound standalone membrane")
    membrane_bake.add_argument("family", choices=FAMILIES, nargs="?")
    membrane_bake.add_argument("--list", action="store_true")
    source = membrane_bake.add_mutually_exclusive_group()
    source.add_argument("--number", help="decimal source N")
    source.add_argument("--input", type=Path, help="file containing decimal N alone")
    source.add_argument("--word-file", type=Path, help="saved IMASM source for ququart or word bakers")
    membrane_bake.add_argument("--output", type=Path, help="fresh output path")
    membrane_bake.add_argument("--mode", help="baked_case mode")
    membrane_bake.add_argument("--list-modes", action="store_true", help="list baked_case modes")
    membrane_bake.add_argument("--width", type=int, help="mode-specific width")
    membrane_bake.add_argument("--radix", type=int, help="divisor mode prefix-closure radix")
    membrane_bake.add_argument("--base", type=int, default=2, help="decimal phase base")
    membrane_bake.add_argument("--prepared", type=Path, help="prepared source directory for CPU baking")
    membrane_bake.add_argument("--line", type=int, default=1, help="source line for word baking")
    modes = membrane_bake.add_mutually_exclusive_group()
    modes.add_argument("--native-fourier", action="store_true")
    modes.add_argument("--retained-case", type=Path)
    sub.add_parser("doctor", help="check local build tools and target availability")
    sub.add_parser("menu", help="choose a program and enter its arguments interactively")
    return p


def main(argv=None):
    p = parser()
    args = p.parse_args(argv)
    args.repo = args.repo.resolve()
    if args.repo not in (VOX, G, M3IOSIS, M3MBR4N3S):
        p.error("Select Vox, G-mOMonadOS, m3iosis or m3mbr4n3s.")
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
            if args.repo not in (VOX, G) and not args.mode:
                raise ValueError("Baking families are available in Vox and G-mOMonadOS.")
            if args.base < 2 or args.line < 1:
                raise ValueError("--base must be at least 2; --line must be positive.")
            bake(args)
        elif args.command == "membranes":
            if args.membrane_action == "list":
                rows = []
                for key, (kind, name, path) in sorted(membrane_catalog(args.repo).items()):
                    if args.query.lower() not in f"{key} {description(path)}".lower():
                        continue
                    action = "build" if kind == "bin" else "run"
                    invocation = f"./programs --repo {shlex.quote(str(args.repo))} membranes {action} {shlex.quote(key)}"
                    if kind == "bin" and baked_number_contract(path):
                        invocation += " --number DECIMAL_N"
                        if baked_number_contract(path) == "MEMBRANE_WORDS":
                            invocation += " --number DECIMAL_M"
                    elif kind == "bin":
                        invocation = cli_example(key, path)
                        invocation = invocation.replace("./programs", f"./programs --repo {shlex.quote(str(args.repo))}", 1)
                    if kind == "script" and path.name == "baked_case.py":
                        invocation = f"./programs --repo {shlex.quote(str(args.repo))} membranes bake --mode divisor --number DECIMAL_N --output ./membranes/{args.repo.name}-divisor"
                    elif kind == "script":
                        invocation = cli_example(key, path)
                        invocation = invocation.replace("./programs", f"./programs --repo {shlex.quote(str(args.repo))}", 1)
                    rows.append({"name": key, "kind": kind, "source": str(path),
                                 "description": description(path),
                                 "flags": display_flags(path), "example": invocation})
                if args.json:
                    print(json.dumps(rows, indent=2))
                else:
                    for row in rows:
                        print(f"{row['name']} [{row['kind']}]  {row['description']}")
                        print(f"  flags: {', '.join(row['flags']) if row['flags'] else 'none'}")
                        if args.examples:
                            print(f"  example: {row['example']}")
                    if not rows:
                        print("No matching entries. Try `membranes list --examples`.")
            elif args.membrane_action == "help":
                show_help(args.repo, args.program)
            elif args.membrane_action == "build":
                kind, name, path = resolve(args.repo, args.program)
                if kind != "bin":
                    raise ValueError("Use `programs bake` to emit source-bound images from Python or shell recipes.")
                crate = next(parent for parent in path.parents if (parent / "Cargo.toml").is_file())
                emitted = binary(crate, name, args, kind)
                print(emitted)
                if args.run:
                    invoke([emitted], args.repo)
            elif args.membrane_action == "run":
                kind, name, path = resolve(args.repo, args.program)
                extra = args.arguments
                contract = baked_number_contract(path) if kind == "bin" else None
                if kind == "script":
                    if path.name == "baked_case.py":
                        raise ValueError("Use `membranes bake --mode MODE --number N` to emit an instance-bound standalone binary; direct baker execution is disabled.")
                    command = [sys.executable if path.suffix == ".py" else "bash", path]
                elif contract:
                    if extra:
                        raise ValueError(f"{name} accepts no runtime arguments. Bake with `membranes build {args.program} --number DECIMAL_N`, then run the emitted binary separately.")
                    crate = next(parent for parent in path.parents if (parent / "Cargo.toml").is_file())
                    target = "x86_64-unknown-linux-musl" if crate == VOX else "x86_64-unknown-linux-gnu"
                    profile = "debug" if args.debug else "release"
                    emitted = crate / "target" / target / profile / name
                    if not emitted.is_file():
                        raise ValueError(f"No baked executable exists at {emitted}. Build it first with `membranes build {args.program} --number DECIMAL_N`.")
                    require_standalone(emitted)
                    command = [emitted]
                else:
                    crate = next(parent for parent in path.parents if (parent / "Cargo.toml").is_file())
                    command = [binary(crate, name, args, kind)]
                command.extend(extra[1:] if extra[:1] == ["--"] else extra)
                print(f"Running standalone: {command[0]}", flush=True)
                process = subprocess.Popen(command, cwd=args.repo, start_new_session=True)
                try:
                    process.communicate()
                except KeyboardInterrupt:
                    os.killpg(process.pid, signal.SIGINT)
                    process.communicate()
                    print("Membrane run interrupted by user.", file=sys.stderr)
                    return 130
                if process.returncode:
                    return process.returncode
            elif args.membrane_action == "bake":
                if args.list_modes:
                    from baked_case import MODES
                    for mode, mode_description in MODES.items():
                        print(f"{mode:16} {mode_description}")
                    return 0
                if args.repo not in (VOX, G) and not args.mode:
                    raise ValueError("Use --mode MODE for the shared source-bound baker in this repository.")
                if args.base < 2 or args.line < 1:
                    raise ValueError("--base must be at least 2; --line must be positive.")
                bake(args)
            else:
                p.error("Choose `membranes list`, `help`, `build` or `run`.")
        elif args.command == "execute":
            execute_image(args.file, args)
        elif args.command == "menu":
            entries = catalog(args.repo)
            repo_name = args.repo.name
            query = ""
            page = 0
            while True:
                rows = render_menu(entries, repo_name, query, page)
                selection = input("\nSelect program (or q): ").strip()
                if selection.lower() in ("q", "quit", "exit"):
                    return 0
                if selection.startswith("/"):
                    query = selection[1:].strip()
                    page = 0
                    continue
                if selection.lower() in ("n", "next"):
                    page += 1
                    continue
                if selection.lower() in ("p", "previous", "prev"):
                    page = max(0, page - 1)
                    continue
                if selection.isdigit():
                    index = int(selection)
                    if index < 1 or index > len(rows):
                        print(f"Choose a number from 1 to {len(rows)}.", file=sys.stderr)
                        continue
                    name = rows[index - 1][0]
                else:
                    name = selection
                try:
                    show_help(args.repo, name)
                    selected = resolve(args.repo, name)
                    if baked_number_contract(selected[2]):
                        prompt = "Decimal inputs for --number (comma-separated; or q to go back): " if baked_number_contract(selected[2]) in ("MEMBRANE_WORDS", "MEMBRANE_DECIMALS") else "Decimal N for --number (or q to go back): "
                        number = input(prompt).strip()
                        if number.lower() in ("q", "quit"):
                            continue
                        run_now = input("Run after baking? Add the explicit --run flag? [y/N]: ").strip().lower()
                        arguments = ["membranes", "build", name]
                        for value in number.split(","):
                            arguments.extend(("--number", value.strip()))
                        if run_now in ("y", "yes"):
                            arguments.append("--run")
                        return main(["--repo", str(args.repo), *( ["--no-build"] if args.no_build else []),
                                     *( ["--debug"] if args.debug else []), *arguments])
                    else:
                        flags, usage = cli_flags(selected[2])
                        print("Flags: " + (", ".join(flags) if flags else "No named CLI flags detected"))
                        if usage:
                            print("Usage: " + " | ".join(usage[:3]))
                        raw_arguments = input("Enter the shown flags and positional values (blank for none, q to go back): ").strip()
                        if raw_arguments.lower() in ("q", "quit"):
                            continue
                        arguments = shlex.split(raw_arguments)
                    return main(["--repo", str(args.repo), *( ["--no-build"] if args.no_build else []),
                                 *( ["--debug"] if args.debug else []), "membranes", "run", name, *arguments])
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
