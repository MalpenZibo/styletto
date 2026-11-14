use super::{Accent, Flavor, Theme, accent};

const NORD_ACCENTS: &[Accent] = &[
    accent("Frost", 0x88c0d0),
    accent("Teal", 0x8fbcbb),
    accent("Blue", 0x81a1c1),
    accent("Deep blue", 0x5e81ac),
    accent("Red", 0xbf616a),
    accent("Orange", 0xd08770),
    accent("Yellow", 0xebcb8b),
    accent("Green", 0xa3be8c),
    accent("Purple", 0xb48ead),
];

pub const NORD: Theme = Theme {
    name: "Nord",
    flavors: &[
        Flavor {
            name: "Polar Night",
            light: false,
            island: 0x2e3440,
            text: 0xeceff4,
            sub: 0xd8dee9,
            dim: 0x616e88,
            success: 0xa3be8c,
            warning: 0xebcb8b,
            danger: 0xbf616a,
            accents: NORD_ACCENTS,
            accent: 0,
        },
        Flavor {
            name: "Snow Storm",
            light: true,
            island: 0xeceff4,
            text: 0x2e3440,
            sub: 0x4c566a,
            dim: 0x616e88,
            success: 0xa3be8c,
            warning: 0xebcb8b,
            danger: 0xbf616a,
            accents: NORD_ACCENTS,
            accent: 3,
        },
    ],
};
