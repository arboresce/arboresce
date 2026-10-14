// Literal contract fixtures. Declarations only; no parser or oracle functions.

pub struct IdCase {
    pub id: &'static str,
    pub input: &'static str,
    pub bytes: Option<[u8; 16]>,
}

pub const ID_CASES: &[IdCase] = &[
    IdCase {
        id: "canonical-example",
        input: "8f3f45e0-8e74-4a18-9b41-3cac1e6bb825",
        bytes: Some([
            0x8f, 0x3f, 0x45, 0xe0, 0x8e, 0x74, 0x4a, 0x18, 0x9b, 0x41, 0x3c, 0xac, 0x1e, 0x6b,
            0xb8, 0x25,
        ]),
    },
    IdCase {
        id: "variant-eight",
        input: "00000000-0000-4000-8000-000000000000",
        bytes: Some([0, 0, 0, 0, 0, 0, 0x40, 0, 0x80, 0, 0, 0, 0, 0, 0, 0]),
    },
    IdCase {
        id: "variant-nine",
        input: "00000000-0000-4000-9000-000000000000",
        bytes: Some([0, 0, 0, 0, 0, 0, 0x40, 0, 0x90, 0, 0, 0, 0, 0, 0, 0]),
    },
    IdCase {
        id: "variant-a",
        input: "00000000-0000-4000-a000-000000000000",
        bytes: Some([0, 0, 0, 0, 0, 0, 0x40, 0, 0xa0, 0, 0, 0, 0, 0, 0, 0]),
    },
    IdCase {
        id: "variant-b",
        input: "00000000-0000-4000-b000-000000000000",
        bytes: Some([0, 0, 0, 0, 0, 0, 0x40, 0, 0xb0, 0, 0, 0, 0, 0, 0, 0]),
    },
    IdCase {
        id: "upper-byte-boundaries",
        input: "ffffffff-ffff-4fff-bfff-ffffffffffff",
        bytes: Some([
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x4f, 0xff, 0xbf, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff,
        ]),
    },
    IdCase {
        id: "empty",
        input: "",
        bytes: None,
    },
    IdCase {
        id: "short",
        input: "8f3f45e0-8e74-4a18-9b41-3cac1e6bb82",
        bytes: None,
    },
    IdCase {
        id: "long",
        input: "8f3f45e0-8e74-4a18-9b41-3cac1e6bb8250",
        bytes: None,
    },
    IdCase {
        id: "uppercase",
        input: "8F3F45E0-8E74-4A18-9B41-3CAC1E6BB825",
        bytes: None,
    },
    IdCase {
        id: "version-one",
        input: "8f3f45e0-8e74-1a18-9b41-3cac1e6bb825",
        bytes: None,
    },
    IdCase {
        id: "version-seven",
        input: "8f3f45e0-8e74-7a18-9b41-3cac1e6bb825",
        bytes: None,
    },
    IdCase {
        id: "version-f",
        input: "8f3f45e0-8e74-fa18-9b41-3cac1e6bb825",
        bytes: None,
    },
    IdCase {
        id: "variant-seven",
        input: "8f3f45e0-8e74-4a18-7b41-3cac1e6bb825",
        bytes: None,
    },
    IdCase {
        id: "variant-c",
        input: "8f3f45e0-8e74-4a18-cb41-3cac1e6bb825",
        bytes: None,
    },
    IdCase {
        id: "variant-f",
        input: "8f3f45e0-8e74-4a18-fb41-3cac1e6bb825",
        bytes: None,
    },
    IdCase {
        id: "nil",
        input: "00000000-0000-0000-0000-000000000000",
        bytes: None,
    },
    IdCase {
        id: "nonhex",
        input: "8g3f45e0-8e74-4a18-9b41-3cac1e6bb825",
        bytes: None,
    },
    IdCase {
        id: "missing-hyphens",
        input: "8f3f45e08e744a189b413cac1e6bb825",
        bytes: None,
    },
    IdCase {
        id: "first-separator",
        input: "8f3f45e0_8e74-4a18-9b41-3cac1e6bb825",
        bytes: None,
    },
    IdCase {
        id: "second-separator",
        input: "8f3f45e0-8e74_4a18-9b41-3cac1e6bb825",
        bytes: None,
    },
    IdCase {
        id: "third-separator",
        input: "8f3f45e0-8e74-4a18_9b41-3cac1e6bb825",
        bytes: None,
    },
    IdCase {
        id: "fourth-separator",
        input: "8f3f45e0-8e74-4a18-9b41_3cac1e6bb825",
        bytes: None,
    },
    IdCase {
        id: "misplaced-separator",
        input: "8f3f45e-08e74-4a18-9b41-3cac1e6bb825",
        bytes: None,
    },
    IdCase {
        id: "braces",
        input: "{8f3f45e0-8e74-4a18-9b41-3cac1e6bb825}",
        bytes: None,
    },
    IdCase {
        id: "urn",
        input: "urn:uuid:8f3f45e0-8e74-4a18-9b41-3cac1e6bb825",
        bytes: None,
    },
    IdCase {
        id: "leading-space",
        input: " 8f3f45e0-8e74-4a18-9b41-3cac1e6bb825",
        bytes: None,
    },
    IdCase {
        id: "trailing-space",
        input: "8f3f45e0-8e74-4a18-9b41-3cac1e6bb825 ",
        bytes: None,
    },
    IdCase {
        id: "newline",
        input: "8f3f45e0-8e74-4a18-9b41-3cac1e6bb825\n",
        bytes: None,
    },
    IdCase {
        id: "carriage-return",
        input: "8f3f45e0-8e74-4a18-9b41-3cac1e6bb825\r",
        bytes: None,
    },
    IdCase {
        id: "tab",
        input: "8f3f45e0-8e74-4a18-9b41-3cac1e6bb825\t",
        bytes: None,
    },
    IdCase {
        id: "nul",
        input: "8f3f45e0-8e74-4a18-9b41-3cac1e6bb825\0",
        bytes: None,
    },
    IdCase {
        id: "nonascii-same-byte-length",
        input: "éf3f45e-8e74-4a18-9b41-3cac1e6bb825",
        bytes: None,
    },
    IdCase {
        id: "unicode-separator",
        input: "8f3f45e0–8e74-4a18-9b41-3cac1e6bb825",
        bytes: None,
    },
];

pub struct BinaryIdCase {
    pub id: &'static str,
    pub bytes: [u8; 16],
    pub canonical: Option<&'static str>,
}

pub const BINARY_ID_CASES: &[BinaryIdCase] = &[
    BinaryIdCase {
        id: "binary-example",
        bytes: [
            0x8f, 0x3f, 0x45, 0xe0, 0x8e, 0x74, 0x4a, 0x18, 0x9b, 0x41, 0x3c, 0xac, 0x1e, 0x6b,
            0xb8, 0x25,
        ],
        canonical: Some("8f3f45e0-8e74-4a18-9b41-3cac1e6bb825"),
    },
    BinaryIdCase {
        id: "binary-low",
        bytes: [0, 0, 0, 0, 0, 0, 0x40, 0, 0x80, 0, 0, 0, 0, 0, 0, 0],
        canonical: Some("00000000-0000-4000-8000-000000000000"),
    },
    BinaryIdCase {
        id: "binary-high",
        bytes: [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x4f, 0xff, 0xbf, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0xff,
        ],
        canonical: Some("ffffffff-ffff-4fff-bfff-ffffffffffff"),
    },
    BinaryIdCase {
        id: "binary-nil",
        bytes: [0; 16],
        canonical: None,
    },
    BinaryIdCase {
        id: "binary-version-one",
        bytes: [0, 0, 0, 0, 0, 0, 0x10, 0, 0x80, 0, 0, 0, 0, 0, 0, 0],
        canonical: None,
    },
    BinaryIdCase {
        id: "binary-version-f",
        bytes: [0, 0, 0, 0, 0, 0, 0xf0, 0, 0x80, 0, 0, 0, 0, 0, 0, 0],
        canonical: None,
    },
    BinaryIdCase {
        id: "binary-variant-seven",
        bytes: [0, 0, 0, 0, 0, 0, 0x40, 0, 0x70, 0, 0, 0, 0, 0, 0, 0],
        canonical: None,
    },
    BinaryIdCase {
        id: "binary-variant-c",
        bytes: [0, 0, 0, 0, 0, 0, 0x40, 0, 0xc0, 0, 0, 0, 0, 0, 0, 0],
        canonical: None,
    },
];
