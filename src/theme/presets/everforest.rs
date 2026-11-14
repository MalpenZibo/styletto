use super::{Flavor, Theme, accent};

pub const EVERFOREST: Theme = Theme {
    name: "Everforest",
    flavors: &[
        Flavor {
            name: "Dark",
            light: false,
            island: 0x232a2e,
            text: 0xd3c6aa,
            sub: 0x9da9a0,
            dim: 0x7a8478,
            success: 0xa7c080,
            warning: 0xdbbc7f,
            danger: 0xe67e80,
            accents: &[
                accent("Green", 0xa7c080),
                accent("Aqua", 0x83c092),
                accent("Blue", 0x7fbbb3),
                accent("Purple", 0xd699b6),
                accent("Yellow", 0xdbbc7f),
                accent("Orange", 0xe69875),
                accent("Red", 0xe67e80),
            ],
            accent: 0,
        },
        Flavor {
            name: "Light",
            light: true,
            island: 0xfdf6e3,
            text: 0x5c6a72,
            sub: 0x829181,
            dim: 0xa6b0a0,
            success: 0x8da101,
            warning: 0xdfa000,
            danger: 0xf85552,
            accents: &[
                accent("Green", 0x8da101),
                accent("Aqua", 0x35a77c),
                accent("Blue", 0x3a94c5),
                accent("Purple", 0xdf69ba),
                accent("Yellow", 0xdfa000),
                accent("Orange", 0xf57d26),
                accent("Red", 0xf85552),
            ],
            accent: 0,
        },
    ],
};
