"""Moves the script-based tweaks (data/tweaks.json) onto the backup/restore engine.

A tweak is converted only when every line of its ON script is something the engine can do
exactly (reg add / reg delete value / sc config start / powercfg value of the active plan).
Its ops get a `d` (used only when no original was saved) taken from what the old OFF script
wrote, so nothing new is assumed. Anything else keeps its scripts and is listed at the end.
Idempotent: run again after editing tweaks.json.  Run: python -I tools/convert_legacy.py
"""
import json, os, re, shlex, sys

HERE = os.path.dirname(os.path.abspath(__file__))
PATH = os.path.join(HERE, "..", "data", "tweaks.json")
HIVES = {"HKEY_LOCAL_MACHINE": "HKLM", "HKEY_CURRENT_USER": "HKCU", "HKEY_CLASSES_ROOT": "HKCR", "HKEY_USERS": "HKU"}
KINDS = {"REG_DWORD": "dword", "REG_SZ": "sz", "REG_EXPAND_SZ": "expand", "REG_BINARY": "binary", "REG_QWORD": "qword", "REG_MULTI_SZ": "multi"}
START = {"disabled": 4, "demand": 3, "auto": 2, "delayed-auto": 2, "boot": 0, "system": 1}
# powercfg -change aliases -> (subgroup, setting, seconds per unit)
CHANGE = {"monitor-timeout": ("SUB_VIDEO", "VIDEOIDLE", 60), "disk-timeout": ("SUB_DISK", "DISKIDLE", 60),
          "standby-timeout": ("SUB_SLEEP", "STANDBYIDLE", 60), "hibernate-timeout": ("SUB_SLEEP", "HIBERNATEIDLE", 60)}


class Skip(Exception):
    pass


def hive(p):
    h, _, rest = p.partition("\\")
    h = HIVES.get(h.upper(), h.upper())
    if h not in ("HKLM", "HKCU", "HKCR", "HKU"):
        raise Skip("hive " + h)
    return h + ("\\" + rest if rest else "")


def num(s):
    return int(s, 16) if s.lower().startswith("0x") else int(s)


def parse(script):
    """-> list of atoms: ('reg', path, name, kind, value) | ('svc', name, start) | ('stop', name) | ('pwr', sub, set, 'ac'|'dc', value)"""
    out = []
    for raw in script.splitlines():
        line = raw.strip()
        low = line.lower()
        if not line or low.startswith(("@echo", "exit /b", "rem ", "::")) or low == "powercfg -setactive scheme_current" or low == "powercfg /setactive scheme_current":
            continue
        try:
            a = shlex.split(line.replace("\\", "\\\\"), posix=True)
        except ValueError:
            raise Skip("unparsable: " + line)
        a = [x.replace("\\\\", "\\") for x in a]
        lo = [x.lower() for x in a]
        if lo[:2] == ["reg", "add"]:
            path = hive(a[2])
            name = a[lo.index("/v") + 1] if "/v" in lo else ("" if "/ve" in lo else None)
            if name is None:
                raise Skip("reg add without value: " + line)
            kind = KINDS[a[lo.index("/t") + 1].upper()] if "/t" in lo else "sz"
            data = a[lo.index("/d") + 1] if "/d" in lo else ""
            val = num(data) if kind in ("dword", "qword") else (data.replace("\\0", "\n") if kind == "multi" else data)
            out.append(("reg", path, name, kind, val))
        elif lo[:2] == ["reg", "delete"]:
            if "/v" not in lo:
                out.append(("delkey", hive(a[2])))
            else:
                out.append(("reg", hive(a[2]), a[lo.index("/v") + 1], None, None))
        elif lo[:2] == ["sc", "start"] or lo[:2] == ["net", "start"]:
            out.append(("start", a[2]))
        elif lo[:2] == ["sc", "config"]:
            rest = " ".join(lo[3:]).replace("start= ", "start=")
            m = re.search(r"start=(\S+)", rest)
            if not m or m.group(1) not in START:
                raise Skip("sc config: " + line)
            out.append(("svc", a[2], START[m.group(1)]))
        elif lo[:2] == ["sc", "stop"] or lo[:2] == ["net", "stop"]:
            out.append(("stop", a[2]))
        elif lo[0] == "powercfg" and lo[1] in ("-setacvalueindex", "/setacvalueindex", "-setdcvalueindex", "/setdcvalueindex"):
            if lo[2] != "scheme_current":
                raise Skip("other power plan: " + line)
            out.append(("pwr", a[3], a[4], "ac" if "ac" in lo[1] else "dc", num(a[5])))
        elif lo[0] == "powercfg" and lo[1] in ("-change", "/change"):
            m = re.fullmatch(r"(.+)-(ac|dc)", lo[2].lstrip("-/"))
            if not m or m.group(1) not in CHANGE:
                raise Skip("powercfg -change: " + line)
            sub, st, mul = CHANGE[m.group(1)]
            out.append(("pwr", sub, st, m.group(2), num(a[3]) * mul))
        else:
            raise Skip("not convertible: " + line)
    return out


def convert(t):
    on, off = parse(t["on"]), parse(t["off"])
    if any(x[0] in ("delkey", "start") for x in on):
        raise Skip("ON deletes a key or starts a service")
    reg_off = {(x[1].lower(), x[2].lower()): x for x in off if x[0] == "reg"}
    # OFF deleted the whole key: every value ON wrote under it was absent before
    for x in on:
        if x[0] == "reg" and any(d[0] == "delkey" and (x[1].lower() + "\\").startswith(d[1].lower() + "\\") for d in off):
            reg_off.setdefault((x[1].lower(), x[2].lower()), ("reg", x[1], x[2], None, None))
    svc_off = {x[1].lower(): x[2] for x in off if x[0] == "svc"}
    pwr_off = {(x[1].lower(), x[2].lower(), x[3]): x[4] for x in off if x[0] == "pwr"}
    ops, pwr = [], {}
    stops = {x[1].lower() for x in on if x[0] == "stop"}
    for x in on:
        if x[0] == "reg":
            _, p, n, k, v = x
            op = {"t": "reg", "p": p, "n": n, "k": k or "dword", "v": v}
            o = reg_off.get((p.lower(), n.lower()))
            if o is not None:
                op["d"] = o[4]  # what the old OFF wrote (None = it deleted the value)
                if k is None and o[3]:
                    op["k"] = o[3]
            ops.append(op)
        elif x[0] == "svc":
            op = {"t": "svc", "n": x[1], "s": x[2], "d": svc_off.get(x[1].lower())}
            if x[1].lower() in stops:
                op["stop"] = True
            ops.append(op)
        elif x[0] == "pwr":
            key = (x[1].lower(), x[2].lower())
            if key not in pwr:
                pwr[key] = {"t": "pwr", "sub": x[1], "set": x[2], "ac": None, "dc": None, "d": None}
                ops.append(pwr[key])
            pwr[key][x[3]] = x[4]
            pwr[key].setdefault("_d", {})[x[3]] = pwr_off.get((key[0], key[1], x[3]))
    for op in ops:
        if op["t"] == "pwr":
            d = op.pop("_d")
            if op["ac"] is None:
                raise Skip("power setting changed on battery only")
            if d.get("ac") is not None and (op["dc"] is None or d.get("dc") is not None):
                op["d"] = [d["ac"], d.get("dc")]
    if stops - {o["n"].lower() for o in ops if o["t"] == "svc"}:
        raise Skip("stops a service without changing its start type")
    if not ops:
        raise Skip("no operations")
    return ops


def main():
    data = json.load(open(PATH, encoding="utf-8"))
    kept, done = [], 0
    for t in data:
        t.pop("ops", None)
        try:
            t["ops"] = convert(t)
            done += 1
        except Skip as e:
            kept.append(f"{t['c']}|{t['id']}: {e}")
    json.dump(data, open(PATH, "w", encoding="utf-8", newline="\n"), indent=1, ensure_ascii=False)
    print(f"converted {done} of {len(data)}; kept as scripts: {len(kept)}")
    for k in kept:
        print("  " + k)


if __name__ == "__main__":
    sys.exit(main())
