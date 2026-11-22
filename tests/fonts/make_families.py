"""Builds families.ttc: a collection of tiny faces, boxes for glyphs, made
here from nothing so the test of `installed_families`' rule needs neither the
system's fonts nor anybody's font vendored. Each face says what the rule
should make of it:

- Fixture Sans, Regular (400) and Bold (700): proportional, no semibold;
- Fixture Mono, Regular (400) and SemiBold (600): monospaced, semibold;
- Fixture Mixed, a monospaced face and a proportional one: not monospaced;
- Fixture Variable, one face with a `wght` axis from 300 to 800: semibold;
- Fixture Caps, an A and no a: left out, as a symbol font would be.

    python3 tests/fonts/make_families.py
"""

from fontTools.fontBuilder import FontBuilder
from fontTools.pens.ttGlyphPen import TTGlyphPen
from fontTools.ttLib.ttCollection import TTCollection


def box(width):
    pen = TTGlyphPen(None)
    pen.moveTo((50, 0))
    pen.lineTo((50, 700))
    pen.lineTo((50 + width, 700))
    pen.lineTo((50 + width, 0))
    pen.closePath()
    return pen.glyph()


def face(family, style, weight, mono=False, letters="aA", wght=None):
    names = {ord(c): f"g{ord(c)}" for c in letters}
    order = [".notdef"] + list(names.values())
    fb = FontBuilder(1000, isTTF=True)
    fb.setupGlyphOrder(order)
    fb.setupCharacterMap(names)
    fb.setupGlyf({name: box(400) for name in order})
    # A monospaced face's glyphs are all as wide; a proportional one's not.
    fb.setupHorizontalMetrics(
        {name: (600 if mono else 500 + 100 * i, 50) for i, name in enumerate(order)}
    )
    fb.setupHorizontalHeader(ascent=800, descent=-200)
    ps = f"{family}-{style}".replace(" ", "")
    fb.setupNameTable({"familyName": family, "styleName": style, "psName": ps})
    fb.setupOS2(
        sTypoAscender=800,
        sTypoDescender=-200,
        usWinAscent=800,
        usWinDescent=200,
        usWeightClass=weight,
    )
    fb.setupPost(isFixedPitch=1 if mono else 0)
    if wght:
        low, high = wght
        fb.setupFvar(axes=[("wght", low, weight, high, "Weight")], instances=[])
    return fb.font


collection = TTCollection()
collection.fonts = [
    face("Fixture Sans", "Regular", 400),
    face("Fixture Sans", "Bold", 700),
    face("Fixture Mono", "Regular", 400, mono=True),
    face("Fixture Mono", "SemiBold", 600, mono=True),
    face("Fixture Mixed", "Regular", 400, mono=True),
    face("Fixture Mixed", "Italic", 400),
    face("Fixture Variable", "Regular", 400, wght=(300, 800)),
    face("Fixture Caps", "Regular", 400, letters="A"),
]
collection.save("tests/fonts/families.ttc")
