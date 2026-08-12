#!/usr/bin/env python3
"""B0001 启发式扫描：同一系统内两个 Query 对同一组件的访问冲突。

规则（Bevy 语义）：
- &mut X = 写；&X / Ref<X> / Option<&X> / Changed<X> / Added<X> = 读
- 冲突：q1 的写集 ∩ q2 的(写∪读)集 非空
- 豁免：两查询经 With/Without 可证不相交（q1 的 With 出现在 q2 的 Without，反之亦然）
- ParamSet 内部的查询由 Bevy 保证互斥，跳过含 ParamSet 的系统（人工看）
"""

import re
import sys
from pathlib import Path

FN_RE = re.compile(r"^(?:pub )?fn ([a-z_0-9]+)\(\n((?:    .*\n)*?)\) (?:->[^{]*)?\{", re.M)


def extract_queries(params: str) -> list[str]:
    """括号配对提取每个 Query<...> 的完整泛型体（正则对嵌套泛型会截断）。"""
    out = []
    i = 0
    while True:
        i = params.find("Query<", i)
        if i < 0:
            break
        j = i + len("Query<")
        depth = 1
        while j < len(params) and depth:
            if params[j] == "<":
                depth += 1
            elif params[j] == ">":
                depth -= 1
            j += 1
        out.append(params[i + len("Query<") : j - 1].strip())
        i = j
    return out


def split_top(s: str) -> list[str]:
    """按顶层逗号切分（忽略尖括号/圆括号内的逗号）。"""
    parts, depth, cur = [], 0, ""
    for ch in s:
        if ch in "<(":
            depth += 1
        elif ch in ">)":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        parts.append(cur.strip())
    return parts


def strip_outer_parens(s: str) -> str:
    s = s.strip()
    if s.startswith("(") and s.endswith(")"):
        return s[1:-1]
    return s


COMP_RE = re.compile(r"^&(?:'\w+ )?(mut )?([A-Za-z_][A-Za-z0-9_:<> ]*?)\s*$")


def parse_access(item: str, writes: set, reads: set, withs: set):
    """数据位的非 Option 组件访问隐含 With 过滤（Bevy 的 archetype 匹配语义），
    必须计入 withs 集合参与不相交判定——否则 `(&Marker, &mut Node)` 系列的
    Without 链会被误判为冲突。"""
    item = item.strip()
    optional = item.startswith("Option<")
    if optional:
        item = item[7:-1].strip()
    if item.startswith("Ref<"):
        comp = item[4:-1].strip()
        reads.add(comp)
        if not optional:
            withs.add(comp)
        return
    if item.startswith("Mut<"):
        comp = item[4:-1].strip()
        writes.add(comp)
        if not optional:
            withs.add(comp)
        return
    m = COMP_RE.match(item)
    if m:
        comp = m.group(2).strip()
        (writes if m.group(1) else reads).add(comp)
        if not optional:
            withs.add(comp)


def parse_filters(s: str, reads: set, withs: set, withouts: set):
    for item in split_top(strip_outer_parens(s)):
        item = item.strip()
        if item.startswith("Or<"):
            parse_filters(item[3:-1], reads, withs, withouts)
        elif item.startswith(("Changed<", "Added<")):
            reads.add(item[item.index("<") + 1 : -1].strip())
        elif item.startswith("With<"):
            withs.add(item[5:-1].strip())
        elif item.startswith("Without<"):
            withouts.add(item[8:-1].strip())


def parse_query(q: str):
    parts = split_top(q)
    writes, reads, withs, withouts = set(), set(), set(), set()
    if parts:
        for item in split_top(strip_outer_parens(parts[0])):
            parse_access(item, writes, reads, withs)
    if len(parts) > 1:
        parse_filters(parts[1], reads, withs, withouts)
    return writes, reads, withs, withouts


def disjoint(q1, q2) -> bool:
    _, _, w1, wo1 = q1
    _, _, w2, wo2 = q2
    return bool(w1 & wo2) or bool(w2 & wo1)


findings = []
for path in sorted(Path("crates").rglob("*.rs")):
    src = path.read_text()
    for fm in FN_RE.finditer(src):
        name, params = fm.group(1), fm.group(2)
        if "ParamSet" in params:
            findings.append((str(path), name, "ParamSet（人工确认内部互斥性即可）", "info"))
            continue
        queries = [parse_query(q) for q in extract_queries(params)]
        for i in range(len(queries)):
            for j in range(i + 1, len(queries)):
                w1, r1, *_ = queries[i]
                w2, r2, *_ = queries[j]
                conflict = (w1 & (w2 | r2)) | (w2 & (w1 | r1))
                if conflict and not disjoint(queries[i], queries[j]):
                    findings.append(
                        (str(path), name, f"组件 {sorted(conflict)} 读写冲突且无 Without 隔离", "CONFLICT")
                    )

hard = [f for f in findings if f[3] == "CONFLICT"]
info = [f for f in findings if f[3] == "info"]
print(f"疑似冲突: {len(hard)}  |  ParamSet 备查: {len(info)}\n")
for path, name, msg, _ in hard:
    print(f"[冲突] {path} :: {name} — {msg}")
print()
for path, name, msg, _ in info:
    print(f"[备查] {path} :: {name} — {msg}")
sys.exit(1 if hard else 0)
