//! The offer family's four pages (REMOTE §15.1; bl-9ea6) — their own table at
//! §12's budget, joined after the standing verbs whose subject (a setting with
//! no conversation under it) these share.

use super::super::{HelpRow, Surface};
use crate::boundary::codec::offer::{APPROVE, DECLINE, OFFER, OFFERS};

pub const OFFER_ROWS: &[HelpRow] = &[
    HelpRow {
        verb: OFFER,
        usage: "/offer <common-name> [foot]",
        summary: "offer a new device a place here: answers one URL, good once, for ten minutes",
        detail: "Mints an offer for a device that will be seated in the focused workspace under \
                 the stated common name, and answers the registration URL — the whole of what \
                 travels to the device, by eye, by paste or as a QR symbol. It carries no key \
                 and no certificate. The device makes its own key pair and claims the URL's code \
                 with the public half; the server signs a leaf for it and both ends then show \
                 one confirmation word. The code is single-use and dies ten minutes after it is \
                 minted; minting again under the same name replaces it. Bare is operator grade; \
                 add `foot` for a tool host. Refused for a name that already has a certificate, \
                 a registration or an unapproved claim, when this box holds no CA, and when its \
                 address names no host a device could dial.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: OFFERS,
        usage: "/offers",
        summary: "the focused workspace's pending offers, each with its state and word",
        detail: "Lists every offer not yet approved or declined. `open` is a code nobody has \
                 claimed; `claimed` is a device that holds the leaf and has not dialled; \
                 `proved` is a device that has dialled with it, which a copy of the URL never \
                 could. A claimed or proved row carries the confirmation word — the six \
                 characters the device is showing.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: APPROVE,
        usage: "/approve <common-name>",
        summary: "seat a proved offer's device in the focused workspace",
        detail: "Registers the offered client in the workspace it was offered, which is the \
                 moment its certificate starts to mean anything: until then every gesture it \
                 sends is refused. Refused unless the offer is proved. Read the word off the \
                 new device and approve the row that matches — that comparison is the whole of \
                 what stops an impostor who claimed the URL first.",
        surface: Surface::Control,
    },
    HelpRow {
        verb: DECLINE,
        usage: "/decline <common-name>",
        summary: "void an offer; one already claimed keeps its name spent",
        detail: "An open offer is forgotten and its name is free again. An offer somebody \
                 already claimed has issued a certificate that cannot be recalled, so declining \
                 it holds that certificate refused for good and spends the name: offer the \
                 device again under another one.",
        surface: Surface::Control,
    },
];
