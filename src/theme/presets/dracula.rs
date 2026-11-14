use super::{Flavor, Theme, accent};

pub const DRACULA: Theme = Theme {
    name: "Dracula",
    flavors: &[Flavor {
        name: "Dracula",
        light: false,
        island: 0x21222c,
        text: 0xf8f8f2,
        // No secondary text colour: this sits between foreground and comment.
        sub: 0xc3c6d9,
        dim: 0x6272a4,
        success: 0x50fa7b,
        warning: 0xf1fa8c,
        danger: 0xff5555,
        accents: &[
            accent("Purple", 0xbd93f9),
            accent("Pink", 0xff79c6),
            accent("Cyan", 0x8be9fd),
            accent("Green", 0x50fa7b),
            accent("Yellow", 0xf1fa8c),
            accent("Orange", 0xffb86c),
            accent("Red", 0xff5555),
        ],
        accent: 0,
    }],
};
