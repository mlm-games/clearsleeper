#!/usr/bin/env python3
"""Check the decomp citations in this port against the reference tree.

Every behaviour change in `src/` carries a citation like `` `d_a_npc_p2.cpp:1049-1051` ``. Those
numbers are written by hand and they drift: a citation gets copied from a neighbour, the
reference gets re-decompiled under it, or a digit is mistyped, and the port then points at a line
that says something else. Reading the wrong line is how a port diverges without anyone deciding
to diverge.

The rule the lint enforces is the one that makes a citation worth anything: the symbol the
comment names has to actually be there. Either the cited lines contain that identifier - the
usual case, a call site or a table row - or the range covers the place the reference defines it.
Anything else is reported, with the lines it does name and the range where it really lives, so
the fix is a lookup rather than a hunt.

Work that cannot be judged is kept out of the failure count: a citation that names no identifier
the reference knows about (`m2CD`, a retail string, a Rust field) has nothing to compare against,
and those show only under `--soft`.

Usage:
    tools/citation_lint.py                        # summary, then every failure
    tools/citation_lint.py --group                # counts per module, worst first
    tools/citation_lint.py --only stale-symbol-range src/sim
    tools/citation_lint.py -q src/sim/npc_p2.rs   # counts for one module
"""

import argparse
import os
import re
import sys
from collections import defaultdict

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
# the reference keeps the implementations in `src` and the declarations in `include`, and a
# citation is a line number in one or the other, so both are part of it
DEFAULT_REFERENCE = os.path.normpath(os.path.join(REPO, "..", "tww"))
REFERENCE_DIRS = ("src", "include")
SRC_EXTENSIONS = (".cpp", ".c", ".h", ".hpp", ".inc")
MAX_PLAUSIBLE_RANGE = 400
TEXT_WINDOW = 3

# `d_npc_event_cut.inc:10-82`, `:387`, `tww/src/d/actor/d_a_npc_p2.cpp:1049`
CITATION = re.compile(
    r"(?:(?P<dir>[A-Za-z0-9_+./-]*/))?(?P<file>[A-Za-z0-9_+.-]+\.(?:cpp|c|h|hpp|inc))?"
    r":(?P<start>\d+)(?:-(?P<end>\d+))?"
)
# what a backticked span claims the citation shows: the identifier it starts with, so
# `cutProc`, `J3DGetKeyFrameInterpolation<s16>` and `dNpc_EventCut_c::cutProc()` all count
SYMBOL = re.compile(r"`([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_~][A-Za-z0-9_]*)*)[^`]*`")
# a claimed name has to look like code, because backticks are also used for prose and for Rust
# fields; `cutProc` and `p_offset` are claims, `stick`, `format` and `animation` are not
LOOKS_LIKE_CODE = re.compile(r"(_|[A-Z].*[A-Za-z0-9]|::)")


def claimed_names(comment):
    return [
        s
        for s in SYMBOL.findall(comment)
        if LOOKS_LIKE_CODE.search(s)
        and s.split("::")[-1] not in KEYWORDS
        and s.split("::")[0] not in DATA_KEYWORDS
    ]


# the file a bare `:NNN` belongs to, wherever the comment mentions it
FILE_MENTION = re.compile(r"([A-Za-z0-9_+.-]+\.(?:cpp|c|h|hpp|inc))")
# a decompiled function name: `head__NNClass...`, where NN is the length of the class name
MANGLED = re.compile(r"^([A-Za-z_~][A-Za-z0-9_]*)__(\d+)([A-Za-z_][A-Za-z0-9_]*)")
# a citation of another Rust file, which points at this workspace or a sibling one and is not the
# decomp's business; blanked out to length so the offsets of the citations around it hold
RUST_CITE = re.compile(r"[A-Za-z0-9_+./-]*\.rs:\d+(?:-\d+)?")


def mask_rust_cites(comment):
    return RUST_CITE.sub(lambda m: " " * len(m.group(0)), comment)

KEYWORDS = {
    "if", "for", "while", "switch", "return", "sizeof", "do", "else", "case", "new", "delete",
    "typedef", "using", "namespace", "class", "struct", "union", "enum", "public", "private",
    "protected", "template", "break", "continue", "default", "throw", "operator",
}
DATA_KEYWORDS = {
    "static", "const", "extern", "volatile", "unsigned", "signed", "register", "constexpr",
    "struct", "union", "enum", "class", "typedef", "template", "inline", "sizeof", "return",
}

# A definition head is `[qualifiers] [return type] name ( params ) {`, and the name is whatever
# sits immediately before the first `(`. Taking it from the tail of the head is exact; no single
# regex gets the return-type boundary right across `bool dNpc_EventCut_c::cutProc()`,
# `void daObjPirateship::Act_c::pirateCreate(int* arg1)` and `s16 calc(void)`.
QUALIFIED_TAIL = re.compile(r"([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_~][A-Za-z0-9_]*)*)\s*$")
# `static const s16 p_offset[][8] = {`, `u32 dr_prm[2];` - a global object, which has no `(`
DATA_TAIL = re.compile(r"([A-Za-z_][A-Za-z0-9_]*)\s*(?:\[[^\]]*\])*\s*(=|;|\{)")


def strip_noise(text):
    """Comments, string and char literals blanked out, one output line per input line.

    Brace counting on raw decomp text is wrong - the address comment blocks and the string
    literals both carry braces - and blanking keeps every line number identical to the file on
    disk, which is what a citation counts against.
    """
    out = []
    in_block = in_str = in_char = False
    for line in text.split("\n"):
        buf = []
        i = 0
        while i < len(line):
            two = line[i : i + 2]
            if in_block:
                if two == "*/":
                    in_block = False
                    i += 2
                else:
                    i += 1
                buf.append(" ")
                continue
            if in_str or in_char:
                quote = '"' if in_str else "'"
                if line[i] == "\\" and i + 1 < len(line):
                    buf.append("  ")
                    i += 2
                    continue
                if line[i] == quote:
                    in_str = in_char = False
                buf.append(" ")
                i += 1
                continue
            if two == "/*":
                in_block = True
                buf.append("  ")
                i += 2
                continue
            if two == "//":
                break
            if line[i] == '"':
                in_str = True
                buf.append(" ")
                i += 1
                continue
            if line[i] == "'":
                in_char = True
                buf.append(" ")
                i += 1
                continue
            buf.append(line[i])
            i += 1
        out.append("".join(buf))
    return out


def func_name(line):
    """The name a definition head defines, or None when the line is not one."""
    head = line.split("(", 1)[0]
    m = QUALIFIED_TAIL.search(head)
    if not m:
        return None
    name = m.group(1)
    return None if name.split("::")[-1] in KEYWORDS else name


def data_name(line):
    """The name a file-scope object declaration declares, or None when the line is not one."""
    stripped = line.strip()
    if "(" in stripped.split("=", 1)[0]:
        return None
    m = DATA_TAIL.search(stripped)
    if not m:
        return None
    name = m.group(1)
    if name in DATA_KEYWORDS or name in KEYWORDS:
        return None
    # only lines that lead with types and storage count, which keeps the statements of a body
    # (`m2CD = 1;`) out of the index
    prefix = stripped.split("=", 1)[0].split("{", 1)[0].split(";", 1)[0].strip()
    if not re.match(r"^[A-Za-z_][A-Za-z0-9_:<>,\s\*&]*$", prefix):
        return None
    return name


def body_end(code, start):
    """The line closing the brace block opened at or after `start`."""
    acc = 0
    opened = False
    for i in range(start, len(code)):
        acc += code[i].count("{") - code[i].count("}")
        if "{" in code[i]:
            opened = True
        if opened and acc <= 0:
            return i
    return len(code) - 1


def init_end(code, start):
    """The terminating line of a global initializer opened at `start`."""
    acc = 0
    opened = False
    for i in range(start, len(code)):
        acc += code[i].count("{") - code[i].count("}")
        if "{" in code[i]:
            opened = True
        if ";" in code[i] and (not opened or acc <= 0):
            return i
    return len(code) - 1


def find_defs(code, allow_depth=0):
    """`[(name, start, end)]` for the definitions at file scope in one blanked-out file.

    `allow_depth` is 1 for headers, whose classes hold their methods inline.
    """
    defs = []
    depth = 0
    i = 0
    n = len(code)
    while i < n:
        line = code[i]
        stripped = line.strip()
        if depth > allow_depth or not stripped or stripped.startswith("#"):
            depth += line.count("{") - line.count("}")
            depth = max(depth, 0)
            i += 1
            continue
        base = depth
        name = func_name(line) if "(" in line else None
        if name is not None:
            if stripped.endswith(";"):
                # a declaration, which is what a citation into `include/` points at
                defs.append((name, i + 1, i + 1))
                i += 1
                continue
            brace_line = i
            if "{" not in line:
                # a multi-line parameter list, or a head whose `{` is on the next line
                probe = i
                while probe + 1 < n and probe - i < 12:
                    probe += 1
                    if "{" in code[probe]:
                        brace_line = probe
                        break
                    if ";" in code[probe]:
                        name = None
                        break
            if name is not None:
                end = body_end(code, brace_line)
                defs.append((name, i + 1, end + 1))
                depth = base
                i = end + 1
                continue
        if name is None and "(" not in line:
            data = data_name(line)
            if data is not None:
                end = init_end(code, i) if "{" in line else i
                defs.append((data, i + 1, end + 1))
                depth = base
                i = end + 1
                continue
        depth += line.count("{") - line.count("}")
        depth = max(depth, 0)
        i += 1
    return defs


def build_index(reference):
    """basename -> [entry], and symbol -> {basenames}, over the reference tree."""
    by_name = defaultdict(list)
    by_symbol = defaultdict(set)
    roots = [
        os.path.join(reference, sub)
        for sub in REFERENCE_DIRS
        if os.path.isdir(os.path.join(reference, sub))
    ]
    for root in roots or [reference]:
        for dirpath, _dirs, files in os.walk(root):
            for f in sorted(files):
                if not f.endswith(SRC_EXTENSIONS):
                    continue
                path = os.path.join(dirpath, f)
                try:
                    raw = open(path, "rb").read()
                except OSError:
                    continue
                text = raw.decode("utf-8", "replace")
                lines = text.split("\n")
                defs = find_defs(strip_noise(text), 1 if f.endswith((".h", ".hpp")) else 0)
                by_name[f].append(
                    {
                        "path": path,
                        "lines": lines,
                        "count": len(lines),
                        "defs": defs,
                        "text": "\n".join(lines),
                    }
                )
                for name, _s, _e in defs:
                    short = name.split("::")[-1]
                    by_symbol[short].add(f)
                    if "::" in name:
                        by_symbol[name].add(f)
                    m = MANGLED.match(short)
                    if m and len(m.group(3)) >= int(m.group(2)):
                        # cutProc__15dNpc_EventCut_cFv is dNpc_EventCut_c::cutProc
                        by_symbol["%s::%s" % (m.group(3)[: int(m.group(2))], m.group(1))].add(f)
    return by_name, by_symbol


NAME_PATTERNS = {}


def name_pattern(short):
    """An identifier match that still finds `cutProc` inside the mangled `cutProc__15dNpc_...`."""
    if short not in NAME_PATTERNS:
        NAME_PATTERNS[short] = re.compile(r"(?<![A-Za-z0-9])%s(?![A-Za-z0-9])" % re.escape(short))
    return NAME_PATTERNS[short]


def comment_blocks(lines):
    """Runs of contiguous comment lines, each [(line_no, text)].

    A citation's file and its symbol both live in the run, so the run is the unit of context.
    """
    blocks = []
    run = []
    for no, line in enumerate(lines, start=1):
        stripped = line.strip()
        if stripped.startswith("//"):
            run.append((no, stripped))
        elif stripped == "":
            continue
        elif run:
            blocks.append(run)
            run = []
    if run:
        blocks.append(run)
    return blocks


def claimed_symbols(comment, block, pos, cut=0, lo=0):
    """The names this comment claims, or the ones the line above it claimed.

    A heading names the function and the citation follows on the next line. The search stops at
    any line that already carries a citation of its own, so a claim is never borrowed twice.

    A citation's own claim is what is written between the previous citation and it: "the refusal at
    `:115`" is about the refusal, while "at `:115`. The parent is `fpc_ProcID`, see `f_op_actor.h:286`"
    introduces fpc_ProcID to the citation that follows, not to `:115`.
    """
    symbols = claimed_names(comment[lo:cut])
    if not symbols:
        symbols = claimed_names(comment)
    if symbols:
        return symbols
    for back in range(pos - 1, max(pos - 4, -1), -1):
        text = block[back][1]
        if CITATION.search(mask_rust_cites(text)):
            break
        found = claimed_names(text)
        if found:
            return found
    return []


def named_in(entry, start, end, symbols):
    """The claimed symbol appearing in the cited lines, or within three lines of them.

    The window is deliberately wider than the citation: an off-by-two line number is untidy, but
    a symbol that appears nowhere near is a citation that has stopped meaning anything, and only
    the second is worth a fix.
    """
    if entry is None or start > entry["count"]:
        return None
    first = max(0, start - 1 - TEXT_WINDOW)
    last = min(entry["count"], end + TEXT_WINDOW)
    body = "\n".join(entry["lines"][first:last])
    for sym in symbols:
        if name_pattern(sym.split("::")[-1]).search(body):
            return sym
    return None


def def_matches(defname, short, cls=None):
    """Whether a definition the indexer found is the symbol the comment names.

    The decomp writes `cutProc__15dNpc_EventCut_cFv` where the class-qualified C++ name would be
    `dNpc_EventCut_c::cutProc`, so both the head of a mangled name and a `Class::` list count.
    """
    tail = defname.split("::")[-1]
    if tail != short and tail.split("__")[0] != short:
        return False
    return cls is None or cls in defname.split("::") or cls in tail


def spans_in(entry, symbols, base):
    """`[(short, [(start, end)])]` for the claimed symbols the file itself defines.

    A bare short name is a weak claim: `load` in `J3DAnmLoader.cpp` is three different members.
    And `J2DPane.cpp:167` written inside the `J2DPane` class is not a stale claim about the line
    the class is declared.
    """
    stem = os.path.splitext(base)[0]
    out = []
    for sym in symbols:
        parts = sym.split("::")
        short = parts[-1]
        cls = parts[-2] if len(parts) > 1 else None
        if cls is None and short == stem:
            continue
        ranges = [(s, e) for name, s, e in entry["defs"] if def_matches(name, short, cls)]
        if ranges:
            out.append((short, ranges))
    return out


def pick_entry(entries, start, end, symbols):
    """Among the files that share a basename, the one a citation could mean."""
    for e in entries:
        if named_in(e, start, end, symbols):
            return e
    for e in entries:
        if end <= e["count"]:
            return e
    return entries[0] if entries else None


def check_one(base, entry, scoped, start, end, symbols, by_symbol, by_name, pool_max, module_files):
    """`[(kind, detail)]` for one citation; `unresolved` is the soft kind that never fails."""
    problems = []
    if end < start:
        return [("reversed-range", "%d < %d" % (end, start))]
    if entry is None:
        if not base:
            return [("unknown-file", "no reference file in scope for :%d" % start)]
        near = sorted(f for f in by_name if base in f or f in base)
        return [
            (
                "unknown-file",
                "%s is not in the reference%s"
                % (base, "; the nearest file is %s" % by_name[near[0]][0]["path"] if near else ""),
            )
        ]
    if len(by_name[base]) > 1:
        problems.append(
            (
                "unresolved",
                "%s matches %s, so the citation does not say which"
                % (base, " and ".join(e["path"] for e in by_name[base])),
            )
        )
    if named_in(entry, start, end, symbols):
        return problems
    for short, ranges in spans_in(entry, symbols, base):
        if any(not (e < start - TEXT_WINDOW or s > end + TEXT_WINDOW) for s, e in ranges):
            return problems
        # a citation the same length as exactly one definition is the same function, numbered
        # against a revision that no longer matches: the fix is the line numbers, not the claim
        same = [r for r in ranges if r[1] - r[0] == end - start]
        fix = same[0] if len(same) == 1 else None
        return [
            (
                "stale-symbol-range",
                "%s is at %s in %s, not at :%d-%d"
                % (short, ", ".join("%d-%d" % (s, e) for s, e in sorted(ranges)[:4]), base, start, end),
                fix,
            )
        ]
    for sym in symbols:
        short = sym.split("::")[-1]
        if short == os.path.splitext(base)[0]:
            continue  # a citation inside a file named after the symbol is not a claim about lines
        where = by_symbol.get(sym) or by_symbol.get(short)
        if not where:
            continue
        if len(where) == 1:
            other = next(iter(where))
            if os.path.splitext(other)[0] == os.path.splitext(base)[0]:
                continue  # a header is the same translation unit as its source
            if name_pattern(short).search(entry["text"]):
                # the cited file uses the symbol even if these lines are not where it is defined:
                # the comment describes a call site, which is legitimate documentation
                continue
            if other in module_files:
                # the module cites both files, so this is a call site described next to the call
                return [
                    (
                        "unresolved",
                        "%s is defined in %s, which this module also cites, at %s:%d-%d"
                        % (short, other, base, start, end),
                    )
                ]
            return [("wrong-file", "%s is defined in %s, not in the cited %s" % (short, other, base))]
        return [
            (
                "unresolved",
                "%s is defined in %d files and appears in none of the cited lines of %s:%d-%d"
                % (short, len(where), base, start, end),
            )
        ]
    if start > entry["count"] or end > entry["count"]:
        if not scoped and start <= pool_max:
            # the file was only guessed, and some file the module does name would have fitted
            return [("unresolved", ":%d-%d is beyond %s, which the comment did not name" % (start, end, base))]
        if start - entry["count"] <= TEXT_WINDOW:
            # a range that runs a line or two off the end of the file is untidy, not wrong
            return [
                (
                    "unresolved",
                    "%s has %d lines, :%d-%d reaches past them" % (base, entry["count"], start, end),
                )
            ]
        return [("past-eof", "%s has %d lines, :%d-%d is beyond it" % (base, entry["count"], start, end))]
    if end - start > MAX_PLAUSIBLE_RANGE:
        return [("implausible-range", "%d lines of %s" % (end - start + 1, base))]
    body = [ln.strip() for ln in entry["lines"][start - 1 : end] if ln.strip()]
    if not body:
        return [("blank-lines", "the cited lines of %s are empty" % base)]
    if all(ln in ("{", "}", "};") for ln in body):
        return [("brace-only", "the cited lines of %s are only braces" % base)]
    if not symbols:
        return [("unresolved", "%s:%d-%d names nothing the reference defines" % (base, start, end))]
    return problems


def header_file(lines, by_name):
    """The reference file a module declares as its own in its leading `//!` block."""
    for line in lines[:60]:
        if not line.strip().startswith("//!"):
            if line.strip():
                return None
            continue
        m = FILE_MENTION.search(line)
        if m and by_name.get(os.path.basename(m.group(1))):
            return os.path.basename(m.group(1))
    return None


def own_file(path, by_name):
    """The reference file a module is named after: `npc_hi1.rs` is `d_a_npc_hi1.cpp`.

    The decomp spells every actor one way, so comparing the two names with the underscores and the
    `d_a_` prefix dropped pairs them without a table to keep up to date.
    """
    def normalize(name):
        stem = os.path.basename(name).rsplit(".", 1)[0]
        return stem.replace("d_a_", "").replace("_", "").lower()

    want = normalize(path)
    if not want:
        return None
    for name in by_name:
        if normalize(name) == want:
            return name
    return None


def cite_field(m):
    """The file part of a citation match, with any directory the comment wrote."""
    if not m.group("file"):
        return None
    return (m.group("dir") or "") + m.group("file")


CANDIDATE_CACHE = {}


def shorthand_candidates(field, by_name):
    """The basenames `main.h` could stand for: `d_a_player_main.h`, `m_Do_main.h`, `main.h`.

    The port cites long decomp names with the front cut off, so a shorthand has to be resolved
    against the files that exist - and a six-line TRK `main.h` is not what line 881 of `main.h`
    means.
    """
    if field not in CANDIDATE_CACHE:
        out = [field] if field in by_name else []
        out += sorted(
            f
            for f in by_name
            if f != field and f.endswith(field) and f[-len(field) - 1 : -len(field)] == "_"
        )
        CANDIDATE_CACHE[field] = out
    return CANDIDATE_CACHE[field]


def resolve_file(field, by_name, module_files, cache, want_end=None):
    """The reference basename a cited file name means, `""` for another workspace, or None.

    The port cites the reference three ways: the basename (`d_a_npc_p2.cpp`), a shorthand for it
    (`main.cpp` for `d_a_player_main.cpp`) and a path (`tww/src/d/actor/d_a_npc_p2.cpp`). A path
    that is not under the reference is some other tree - the Rust workspace, a sibling crate - and
    is not a decomp citation at all.
    """
    if field in cache:
        return cache[field]
    out = None
    if "/" in field:
        want = field if not field.startswith("tww/") else field[len("tww/") :]
        hits = {
            os.path.basename(e["path"])
            for entries in by_name.values()
            for e in entries
            if e["path"].replace(os.sep, "/").endswith(want)
        }
        out = hits.pop() if len(hits) == 1 else ""
        cache[field] = out
    else:
        cands = shorthand_candidates(field, by_name)
        if len(cands) == 1:
            out = cands[0]
            cache[field] = out
        elif cands:
            if want_end:
                fits = [c for c in cands if max(e["count"] for e in by_name[c]) >= want_end]
                if fits:
                    cands = fits
            known = sorted({c for c in cands if c in module_files})
            if len(known) == 1:
                out = known[0]
            elif len(cands) == 1:
                out = cands[0]
            if want_end is None and out:
                cache[field] = out
    return out


def module_citations(lines, by_name, alias):
    """The reference files a module names, most-cited first, with shorthands resolved."""
    counts = defaultdict(int)
    for line in lines:
        for m in CITATION.finditer(mask_rust_cites(line)):
            field = cite_field(m)
            if not field:
                continue
            got = resolve_file(
                field, by_name, frozenset(), alias, int(m.group("end") or m.group("start"))
            )
            if got:
                counts[got] += 1
    return sorted(counts, key=lambda f: (-counts[f], f))


def check_file(path, by_name, by_symbol, show_all=False):
    text = open(path, "rb").read().decode("utf-8", "replace")
    lines = text.split("\n")
    rel = os.path.relpath(path, REPO)
    alias = {}
    mod_files = module_citations(lines, by_name, alias)
    module_files = frozenset(mod_files)
    primary = header_file(lines, by_name) or own_file(path, by_name) or (mod_files[0] if mod_files else None)
    pool_max = max([by_name[f][0]["count"] for f in mod_files] or [0])
    def judge(base, start, end, symbols, scoped, explicit):
        """Which reference file a citation could mean, and what is wrong with that reading."""
        entry = pick_entry(by_name.get(base, []), start, end, symbols)
        if not scoped and not named_in(entry, start, end, symbols):
            # a bare `:NNN` belongs to the file the block last named; when the block named
            # nothing, take whichever file the module cites holds the claimed symbol there
            for cand in mod_files + ([primary] if primary and primary not in mod_files else []):
                candidate = pick_entry(by_name.get(cand, []), start, end, symbols)
                if named_in(candidate, start, end, symbols):
                    entry, base = candidate, cand
                    break
        problems = check_one(
            base, entry, scoped, start, end, symbols, by_symbol, by_name, pool_max, module_files
        )
        if not explicit and start > (entry["count"] if entry else 0):
            # a bare range that overruns the file the block last named usually belongs to
            # one of the other files the block named
            for cand in reversed(named):
                if cand == base or cand not in by_name:
                    continue
                candidate = pick_entry(by_name.get(cand, []), start, end, symbols)
                if named_in(candidate, start, end, symbols) or (
                    candidate
                    and candidate["count"] >= end
                    and not check_one(
                        cand,
                        candidate,
                        scoped,
                        start,
                        end,
                        symbols,
                        by_symbol,
                        by_name,
                        pool_max,
                        module_files,
                    )
                ):
                    entry, base = candidate, cand
                    problems = check_one(
                        base,
                        entry,
                        scoped,
                        start,
                        end,
                        symbols,
                        by_symbol,
                        by_name,
                        pool_max,
                        module_files,
                    )
                    break
        return base, entry, problems

    results = []
    checked = 0
    # every file the module has named, most recent last: a bare range that overruns the file the
    # comment was talking about usually belongs to one of the others it named nearby
    named = []
    for block in comment_blocks(lines):
        scope = None
        for pos, (no, comment) in enumerate(block):
            scan = mask_rust_cites(comment)
            prev_end = 0
            mention = FILE_MENTION.search(scan)
            if mention:
                scope = (
                    resolve_file(mention.group(1), by_name, module_files, alias)
                    or mention.group(1)
                )
            extern = bool(RUST_CITE.search(comment)) or any(
                cite_field(other) and resolve_file(cite_field(other), by_name, module_files, alias) == ""
                for other in CITATION.finditer(scan)
            )
            for m in CITATION.finditer(scan):
                start = int(m.group("start"))
                end = int(m.group("end")) if m.group("end") else start
                field = cite_field(m)
                if field:
                    resolved = resolve_file(field, by_name, module_files, alias, end)
                    if resolved == "":
                        continue  # a citation of another tree, not of the reference
                    if resolved is None:
                        resolved = os.path.basename(field)
                    scope = resolved
                    explicit = resolved
                else:
                    explicit = None
                if scope and scope not in named:
                    named.append(scope)
                checked += 1
                scoped = bool(explicit or scope)
                base = explicit or scope or primary or ""
                symbols = claimed_symbols(scan, block, pos, m.start(), prev_end)
                prev_end = m.end()
                base, entry, problems = judge(base, start, end, symbols, scoped, explicit)
                if any(p[0] != "unresolved" for p in problems):
                    # a shorthand that several reference files answer to, or a clause whose own
                    # claim was too thin to verify, is worth reading the generous way before it is
                    # called a broken citation
                    trials = [(base, symbols + [s for s in claimed_names(scan) if s not in symbols])]
                    if explicit:
                        trials += [(alt, symbols) for alt in shorthand_candidates(field, by_name)
                                   if alt != base]
                    for alt_base, alt_symbols in trials:
                        if len(alt_symbols) == len(symbols) and alt_base == base:
                            continue
                        b, e, alt = judge(alt_base, start, end, alt_symbols, scoped, explicit)
                        if len([p for p in alt if p[0] != "unresolved"]) < len(
                            [p for p in problems if p[0] != "unresolved"]
                        ):
                            base, entry, problems = b, e, alt
                for problem in problems:
                    kind, detail = problem[0], problem[1]
                    fix = problem[2] if len(problem) > 2 else None
                    if kind != "unresolved" and not scoped and extern:
                        # a bare range beside a citation of another tree may be pointing there
                        kind = "unresolved"
                        detail += "; the same comment cites outside the reference"
                        fix = None
                    results.append((rel, no, kind, comment, detail, fix, m.span()))
                if show_all and not problems:
                    print("ok   %s:%d %s" % (rel, no, comment))
    return results, checked


def rust_sources(paths):
    out = []
    for p in paths or [os.path.join(REPO, "src")]:
        p = os.path.abspath(p)
        if os.path.isdir(p):
            out += sorted(
                os.path.join(r, f)
                for r, _d, fs in os.walk(p)
                for f in fs
                if f.endswith(".rs")
            )
        else:
            out.append(p)
    return out


def apply_fixes(findings, write):
    """Renumber the citations the reference pins down exactly, and report how many moved.

    Only findings that carry a fix do: the comment's own claim was found as a single definition of
    the same length as the cited range, so the range is the right function at the wrong numbers.
    """
    per_file = defaultdict(list)
    for f in findings:
        if f[5]:
            per_file[f[0]].append((f[1], f[6][0], f[5]))
    moved = 0
    for rel, items in sorted(per_file.items()):
        path = os.path.join(REPO, rel)
        lines = open(path, "rb").read().decode("utf-8", "replace").split("\n")
        touched = 0
        for no, offset, (start, end) in sorted(items, key=lambda i: (-i[0], -i[1])):
            original = lines[no - 1]
            lead = len(original) - len(original.lstrip())
            found = CITATION.search(original, offset + lead)
            if not found or found.start() != offset + lead:
                continue
            old = original[found.start() : found.end()]
            new = re.sub(
                r"\d+(?:-\d+)?$", "%d-%d" % (start, end) if start != end else "%d" % start, old
            )
            if new == old:
                continue
            lines[no - 1] = original[: found.start()] + new + original[found.end() :]
            moved += 1
            touched += 1
        if write and touched:
            open(path, "w", encoding="utf-8").write("\n".join(lines))
    return moved


def main():
    ap = argparse.ArgumentParser(description="check the decomp citations in this port")
    ap.add_argument("paths", nargs="*", help="Rust files or directories (default: src)")
    ap.add_argument("--reference", default=DEFAULT_REFERENCE, help="the decomp tree")
    ap.add_argument("--all", action="store_true", help="print the citations that pass too")
    ap.add_argument("--soft", action="store_true", help="report citations nothing can confirm")
    ap.add_argument("--group", action="store_true", help="counts per module instead of each finding")
    ap.add_argument("--only", action="append", default=[], help="just these kinds (repeatable)")
    ap.add_argument("--limit", type=int, default=0, help="stop after N findings")
    ap.add_argument("-q", "--quiet", action="store_true", help="summary only")
    ap.add_argument(
        "--fix", action="store_true", help="renumber the citations the reference pins down exactly"
    )
    ap.add_argument("--dry-run", action="store_true", help="with --fix: report, do not write")
    args = ap.parse_args()

    paths = rust_sources(args.paths)
    by_name, by_symbol = build_index(args.reference)
    print(
        "reference: %d files, %d defined names" % (len(by_name), len(by_symbol))
    )

    results = []
    citations = 0
    for path in paths:
        found, checked = check_file(path, by_name, by_symbol, args.all)
        results += found
        citations += checked

    soft = [f for f in results if f[2] == "unresolved"]
    hard = [f for f in results if f[2] != "unresolved"]
    # --only and --limit choose what is printed; they never change what the run reports or fails on
    shown = hard + (soft if args.soft else [])
    if args.only:
        wanted = set(args.only)
        shown = [f for f in shown if f[2] in wanted]
    if args.limit:
        shown = shown[: args.limit]

    failed = {(f[0], f[1], f[3]) for f in hard}
    print(
        "%d citations in %d files: %d confirmed, %d failed, %d nothing can confirm"
        % (citations, len(paths), citations - len(failed) - len(soft), len(hard), len(soft))
    )
    kinds = defaultdict(int)
    for f in hard:
        kinds[f[2]] += 1
    for kind, count in sorted(kinds.items(), key=lambda kv: -kv[1]):
        print("  %-20s %d" % (kind, count))
    if args.group:
        per_file = defaultdict(lambda: defaultdict(int))
        for f in hard:
            per_file[f[0]][f[2]] += 1
        for rel in sorted(per_file, key=lambda r: -sum(per_file[r].values())):
            print("%-34s %s" % (rel, dict(per_file[rel])))
        return 1 if hard else 0
    if not args.quiet:
        if len(shown) < len(hard) + len(soft):
            print("showing %d of them" % len(shown))
        for f in shown:
            print("%s:%d  %-18s %s" % (f[0], f[1], f[2], f[3][:160]))
            print("  -> %s" % f[4][:220])
    if args.fix:
        moved = apply_fixes(hard, not args.dry_run)
        print("renumbered %d citations%s" % (moved, " (dry run)" if args.dry_run else ""))
    return 1 if hard else 0


if __name__ == "__main__":
    sys.exit(main())
