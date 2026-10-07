//! Randomness: numbers, picks, shuffles, UUIDs.

use crate::interp::Interp;
use crate::lexer::Span;
use crate::modules::{Call, Doc, Fail, Module, Section, doc, text::encoding::hex};
use crate::value::{Value, num};

pub const MODULE: Module = Module {
    name: "random",
    about: "numbers, picks, shuffles, UUIDs; passwords, passphrases, ULIDs and nano IDs from the OS's secure generator",
    examples: EXAMPLES,
    fns: FNS,
    call,
    ..Module::EMPTY
};

#[rustfmt::skip]
const EXAMPLES: &[Section] = &[
    ("random", &[
        ("roll a die", "rand(1, 6)"),
        ("roll three", r#"(1..=3).map(|_| rand(1, 6))"#),
        ("float in a range", "rand(1.0, 2.0)"),
        ("pick one", r#"["rock", "paper", "scissors"].choice"#),
        ("shuffle", "(1..=5).shuffle"),
        ("UUID", "uuid()"),
        ("a strong password", "password()"),
        ("one you can type", "passphrase()"),
        ("sortable IDs", "ulid()"),
    ]),
];

#[rustfmt::skip]
const FNS: &[Doc] = &[
    doc("rand", "rand() / rand(a: num, b: num)", "float in [0, 1), or a number from a to b inclusive", &["rand()", "rand(1, 6)"], &["choice", "shuffle"]),
    doc("choice", "choice(xs: list)", "random item", &[r#"["rock", "paper", "scissors"].choice"#], &["rand", "shuffle"]),
    doc("shuffle", "shuffle(xs: list)", "shuffled copy", &["(1..6).shuffle"], &["choice"]),
    doc("uuid", "uuid()", "random v4 UUID", &["uuid()"], &["rand", "ulid"]),
    doc("password", "password(len?: int)", "a password of letters, digits and symbols (default 20 characters, about 6.5 bits each), from the OS's secure generator", &["password()", "password(12)"], &["passphrase"]),
    doc("passphrase", "passphrase(words?: int)", "random words joined by dashes (default 6, about 9.4 bits per word), from the OS's secure generator", &["passphrase()", "passphrase(4)"], &["password"]),
    doc("ulid", "ulid()", "a ULID: 26 characters that sort by creation time, then 80 random bits", &["ulid()"], &["uuid", "nanoid"]),
    doc("nanoid", "nanoid(len?: int)", "a URL-safe random ID (default 21 characters, like a UUID's strength)", &["nanoid()", "nanoid(8)"], &["uuid", "ulid"]),
];

fn call(_: &mut Interp, name: &'static str, args: &[Value], _: &Span) -> Call {
    use Value::*;
    Ok(match (name, args) {
        ("rand", []) => Float(fastrand::f64()),
        ("rand", [Int(a, _), Int(b, _)]) if a <= b => Value::int(fastrand::i64(*a..=*b)),
        ("rand", [a, b]) if num(a).is_some() && num(b).is_some() => {
            let (a, b) = (num(a).unwrap(), num(b).unwrap());
            Float(a + fastrand::f64() * (b - a))
        }
        ("choice", [List(l)]) => {
            let l = l.borrow();
            if l.is_empty() {
                return Err("empty list".into());
            }
            l[fastrand::usize(..l.len())].clone()
        }
        ("shuffle", [List(l)]) => {
            let mut v = l.borrow().clone();
            fastrand::shuffle(&mut v);
            Value::list(v)
        }
        ("uuid", []) => {
            let mut b: [u8; 16] = std::array::from_fn(|_| fastrand::u8(..));
            b[6] = (b[6] & 0x0f) | 0x40; // version 4
            b[8] = (b[8] & 0x3f) | 0x80; // RFC 4122 variant
            let h = hex(&b);
            Value::str(format!("{}-{}-{}-{}-{}", &h[..8], &h[8..12], &h[12..16], &h[16..20], &h[20..]))
        }
        ("password", []) => secret(20, PASSWORD_CHARS)?,
        ("password", [Int(n, _)]) if (4..=1000).contains(n) => secret(*n as usize, PASSWORD_CHARS)?,
        ("nanoid", []) => secret(21, NANOID_CHARS)?,
        ("nanoid", [Int(n, _)]) if (1..=1000).contains(n) => secret(*n as usize, NANOID_CHARS)?,
        ("password" | "nanoid", [Int(n, _)]) => {
            return Err(Fail::Arg(0, format!("length must be from {} to 1000, got {n}", if name == "password" { 4 } else { 1 })));
        }
        ("passphrase", []) => passphrase(6)?,
        ("passphrase", [Int(n, _)]) if (1..=100).contains(n) => passphrase(*n as usize)?,
        ("passphrase", [Int(n, _)]) => return Err(Fail::Arg(0, format!("word count must be from 1 to 100, got {n}"))),
        ("ulid", []) => {
            let ms = jiff::Timestamp::now().as_millisecond() as u128;
            let mut r = [0u8; 10];
            getrandom::getrandom(&mut r).map_err(|e| e.to_string())?;
            let n = (ms << 80) | r.iter().fold(0u128, |acc, b| acc << 8 | *b as u128);
            Value::str((0..26).rev().map(|i| CROCKFORD[(n >> (i * 5)) as usize & 31] as char).collect::<String>())
        }
        _ => return Err(Fail::BadArgs),
    })
}

const PASSWORD_CHARS: &str = "ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789!@#$%^&*-_=+?";
const NANOID_CHARS: &str = "useandom-26T198340PX75pxJACKVERYMINDBUSHWOLF_GQZbfghjklqvwyzrict";
const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// Uniform secure index below `n`, by rejection so no index is favored.
fn secure_index(n: usize) -> Result<usize, String> {
    let limit = u16::MAX as usize + 1;
    let cut = limit - limit % n;
    loop {
        let mut b = [0u8; 2];
        getrandom::getrandom(&mut b).map_err(|e| e.to_string())?;
        let x = u16::from_le_bytes(b) as usize;
        if x < cut {
            return Ok(x % n);
        }
    }
}

fn secret(len: usize, alphabet: &str) -> Result<Value, String> {
    let cs: Vec<char> = alphabet.chars().collect();
    Ok(Value::str((0..len).map(|_| secure_index(cs.len()).map(|i| cs[i])).collect::<Result<String, _>>()?))
}

fn passphrase(n: usize) -> Result<Value, String> {
    let words: Vec<&str> = WORDS.split_whitespace().collect();
    Ok(Value::str((0..n).map(|_| secure_index(words.len()).map(|i| words[i])).collect::<Result<Vec<_>, _>>()?.join("-")))
}

/// Short, common, distinct words for passphrases.
const WORDS: &str = "acid acorn actor adapt adobe agent alarm album alert alias alien alley alpha amber amigo ample angel anger angle ankle apple april apron arena argue \
armor arrow aspen atlas atom attic audio avoid awake award bacon badge bagel baker balmy banjo barge basil batch beach beard beast begin bench berry \
bias bike bingo birch bison blade blank blast blaze blend blimp blink bliss block blond bloom blues bluff blunt board bonus boost booth boots bored \
bossy bound boxer brain brake brand brass brave bread brick bride brief brine brisk broom brush buddy buggy bulb bunch bunny cabin cable cacao cadet \
camel camera candy canoe canon cargo carol carve cedar chain chalk champ chant chaos charm chart chase cheek cheer chess chief child chili chimp chirp \
choir chomp chord chunk cider cinch civic claim clamp clash clasp class clay clerk click cliff climb cloak clock cloud clown coach coast cobra cocoa \
comet comic coral couch cough crane crate crawl crazy cream crest crisp crown crumb crust cubic cupid curly curry cycle daisy dance dandy decal decoy \
delta denim depot diary dice diner disco ditch diver dizzy dodge dolly donut dough dozen draft drama dream dress drift drill drink drive drone drum \
dusty eagle easel ebony eclair edge eject elbow elder elect elite elm ember emoji empty enjoy entry envoy equal error essay ethic event exact exile \
extra fable facet fairy faith false fancy feast fence ferry fever fiber field fifty final finch fjord flair flake flame flash flask fleet flint float \
flock flood floor flour fluid flute focal focus foggy forge forum fossil frame frost fruit fudge funky fuzzy gamer gauge gecko genie ghost giant giddy \
ginger glade gland glass gleam glide globe glove gnome golf goose gorge grace grain grape graph grasp gravy great greed grief grill grind groan groom \
grove growl guard guava guest guide guild gummy gusto habit hairy happy hardy harsh hasty haven hazel heart heavy hedge hefty helix hello heron hippo \
hobby holly honey hoop horse hotel house humid humor hurry husky hyena icing icon ideal igloo image index inlet input ivory jazzy jelly jewel jiffy \
jockey joker jolly judge juice jumbo jumpy kayak kebab kettle kiosk kitty knack knife knock koala label lace ladder lake lamp lance laser latch lava \
lemon level lilac limbo linen lion llama lobby lodge logic lotus lucky lunar lunch lyric macro magic mango manor maple march mask match mayor medal \
melon mercy merry metal meter mimic minty mocha model mojo money moose motor mound mouse movie muddy mural music nacho nadir naval neon nerve never \
ninja noble noise north notch novel nudge nutty oasis ocean olive omega onion opera orbit organ otter ounce outer oxide ozone paddy panda panel panic \
paper parka party pasta patch peach pearl pecan pedal penny peony perch piano pilot pinch pixel pizza plaid plank plaza plume plump poem polar polka \
poppy porch pouch power prism prize prune pulse puppy purse quack quail quart queen quest quick quiet quilt quirk quota rabbit radar radio rainy rally \
ranch raven razor rebel relay remix rhino rhyme ridge rival river roast robin robot rocky rodeo rogue roomy roost rover royal ruby rugby rumba rusty \
saber salad salsa salty sauna scale scarf scone scout scrap scuba sedan shark sheep shelf shell shiny shrub siren sixty skate skunk slate sleek slice \
slope sloth smile smoky snack snail snowy solar sonic spade spark spice spoon sport squid stack stamp steam stork storm stove sugar sunny super swamp \
swift syrup table taco talon tango tapir tasty teddy tempo thorn tiger toast token topaz torch totem tower track trail train trend tribe trout truck \
tulip tuna tunic turbo tweed twist ultra umbra uncle union unity urban usher vague valve vapor vault velvet venom venue verse video vigor vinyl viola \
viper visor vivid vocal voter wafer wagon waltz waste water whale wheat whisk windy witch wizard woody world woven wrath yacht yeast yodel yummy zebra \
zesty zippy zonal";

#[cfg(test)]
mod tests {
    use crate::interp::tests::{eval, try_eval};

    #[test]
    fn random() {
        assert!(matches!(eval("rand(1, 6)"), crate::value::Value::Int(1..=6, _)));
        assert_eq!(eval("uuid().len").to_string(), "36");
        assert_eq!(eval("[1, 2, 3].shuffle.sort").to_string(), "[1, 2, 3]");
        assert!(try_eval("choice([])").is_err());
        let words: Vec<_> = super::WORDS.split_whitespace().collect();
        assert_eq!(words.len(), 653);
        assert_eq!(words.iter().collect::<std::collections::HashSet<_>>().len(), words.len(), "duplicate passphrase words");
        assert_eq!(eval("[password().len, password(8).len, nanoid().len, ulid().len, passphrase(4).split(\"-\").len]").to_string(), "[20, 8, 21, 26, 4]");
        assert!(try_eval("password(3)").is_err());
    }
}
