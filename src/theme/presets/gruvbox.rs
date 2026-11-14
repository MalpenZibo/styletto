use super::{Flavor, Theme, accent};

pub const GRUVBOX: Theme = Theme {
    name: "Gruvbox",
    flavors: &[
        Flavor {
            name: "Dark",
            light: false,
            island: 0x1d2021,
            text: 0xebdbb2,
            sub: 0xbdae93,
            dim: 0x928374,
            success: 0xb8bb26,
            warning: 0xfabd2f,
            danger: 0xfb4934,
            accents: &[
                accent("Blue", 0x83a598),
                accent("Aqua", 0x8ec07c),
                accent("Green", 0xb8bb26),
                accent("Yellow", 0xfabd2f),
                accent("Orange", 0xfe8019),
                accent("Red", 0xfb4934),
                accent("Purple", 0xd3869b),
            ],
            accent: 0,
        },
        Flavor {
            name: "Light",
            light: true,
            island: 0xf9f5d7,
            text: 0x3c3836,
            sub: 0x665c54,
            dim: 0x7c6f64,
            success: 0x79740e,
            warning: 0xb57614,
            danger: 0x9d0006,
            accents: &[
                accent("Blue", 0x076678),
                accent("Aqua", 0x427b58),
                accent("Green", 0x79740e),
                accent("Yellow", 0xb57614),
                accent("Orange", 0xaf3a03),
                accent("Red", 0x9d0006),
                accent("Purple", 0x8f3f71),
            ],
            accent: 0,
        },
    ],
};
