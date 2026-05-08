#!/usr/bin/env python3
"""Lokale sidecar voor [openai/privacy-filter](https://huggingface.co/openai/privacy-filter).

Voorbereiding::

    pip install -r scripts/requirements-privacy-filter.txt
    uvicorn scripts.privacy_filter_server:app --host 127.0.0.1 --port 8091

Of vanuit deze map::

    cd scripts && uvicorn privacy_filter_server:app --host 127.0.0.1 --port 8091

Daarna bij agent-service::

    export PII_PRIVACY_FILTER_URL=http://127.0.0.1:8091

Zie ook de HF-modelkaart (**Bias, Risks, and Limitations**): géén volledige
anonymisatie- of compliancegarantie.
"""

from __future__ import annotations

import os
from functools import lru_cache
from typing import Any

from fastapi import FastAPI
from pydantic import BaseModel, Field

MODEL_ID = os.environ.get("PII_MODEL_ID", "openai/privacy-filter")

# Labels volgens HF-modelkaart voor openai/privacy-filter
ALL_HF_GROUPS: frozenset[str] = frozenset(
    {
        "account_number",
        "private_address",
        "private_email",
        "private_person",
        "private_phone",
        "private_url",
        "private_date",
        "secret",
    }
)

# Veldnamen zoals Rust `PIICategory` → mengformaat van het HF-model
RUST_TO_HF: dict[str, frozenset[str]] = {
    "Email": frozenset({"private_email"}),
    "PhoneNumber": frozenset({"private_phone"}),
    "Name": frozenset({"private_person"}),
    "Address": frozenset({"private_address"}),
    "SSN": frozenset({"account_number", "secret"}),
    "IBAN": frozenset({"account_number", "secret"}),
    # Brede dekking bij custom regels uit de gebruiker
    "Custom": ALL_HF_GROUPS,
}


class MaskIn(BaseModel):
    content: str
    categories: list[str] = Field(default_factory=list)


class EntityOut(BaseModel):
    entity_group: str
    score: float | None = None
    word: str | None = None


class MaskOut(BaseModel):
    masked: str
    entities: list[EntityOut]


def normalize_label(entity_group: str) -> str:
    s = entity_group.strip().lower()
    for pref in ("b-", "i-", "o-", "e-", "s-"):
        if s.startswith(pref):
            s = s[len(pref) :]
            break
    return s.replace("-", "_")


def hf_labels_for_rust_categories(categories: list[str]) -> frozenset[str] | None:
    """None = geen filter: masker elke door het model gevonden span."""
    if not categories:
        return None
    acc: set[str] = set()
    for raw in categories:
        key = raw.strip()
        if key.startswith("Custom"):
            acc |= ALL_HF_GROUPS
            continue
        part = RUST_TO_HF.get(key)
        if part is not None:
            acc |= part
        else:
            acc |= ALL_HF_GROUPS
    return frozenset(acc) if acc else frozenset()


@lru_cache(maxsize=1)
def _pipe():
    from transformers import pipeline

    return pipeline(
        "token-classification",
        model=MODEL_ID,
        aggregation_strategy="simple",
        device_map=os.environ.get("PII_TRANSFORMERS_DEVICE_MAP", "auto"),
    )


app = FastAPI(title="openai/privacy-filter sidecar")


@app.get("/healthz")
def healthz() -> dict[str, str]:
    return {"status": "ok", "model": MODEL_ID}


def _merge_intervals(
    spans: list[tuple[int, int, str, float | None]],
) -> list[tuple[int, int, str, float | None]]:
    if not spans:
        return []
    sp = sorted(spans, key=lambda x: x[0])
    cur = list(sp[0])
    out: list[tuple[int, int, str, float | None]] = []
    for s, e, g, sc in sp[1:]:
        if s <= cur[1]:
            cur[1] = max(cur[1], e)
            # behoud label van eerste span; score onduidelijk bij merge
        else:
            out.append((cur[0], cur[1], cur[2], cur[3]))
            cur = [s, e, g, sc]
    out.append((cur[0], cur[1], cur[2], cur[3]))
    return out


@app.post("/mask", response_model=MaskOut)
def mask(req: MaskIn) -> MaskOut:
    text = req.content or ""
    allowed = hf_labels_for_rust_categories(req.categories)

    pipe = _pipe()
    ents: list[dict[str, Any]] = pipe(text) if text.strip() else []

    raw_spans: list[tuple[int, int, str, float | None]] = []
    for e in ents:
        grp = normalize_label(str(e.get("entity_group", "")))
        if allowed is not None and grp not in allowed:
            continue
        try:
            start = int(e["start"])
            end = int(e["end"])
        except (KeyError, TypeError, ValueError):
            continue
        if end <= start or start < 0 or end > len(text):
            continue
        sc = e.get("score")
        scf = float(sc) if sc is not None else None
        raw_spans.append((start, end, grp, scf))

    merged_spans = _merge_intervals(raw_spans)

    redact = [False] * len(text)
    out_entities: list[EntityOut] = []
    for start, end, grp, _sc in merged_spans:
        for i in range(start, end):
            redact[i] = True
        out_entities.append(EntityOut(entity_group=grp, score=_sc, word=text[start:end]))

    parts: list[str] = []
    i = 0
    while i < len(text):
        if not redact[i]:
            parts.append(text[i])
            i += 1
            continue
        parts.append("[REDACTED]")
        while i < len(text) and redact[i]:
            i += 1

    masked = "".join(parts)

    return MaskOut(masked=masked, entities=out_entities)
