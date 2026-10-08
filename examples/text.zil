# Run: zil examples/text.zil

quote = "The quick brown fox jumps over the lazy dog"
letters = quote.lower.chars.filter(|c| c != " ").unique
print("pangram: {letters.len == 26}")
print(quote.words.map(|w| [w, w.len]).from_entries.bars)

# Case styles
for s in ["user id", "HTTP response code"] { print("  {s.snake}  {s.camel}  {s.kebab}") }

# Ciphers
secret = "attack at dawn".caesar(3)
print("caesar: {secret} -> {secret.caesar(-3)}")
print("rot13:  {"Hello".rot13}")
print("morse:  {"sos".morse}")

# Fuzzy matching
commands = ["commit", "checkout", "cherry-pick", "clone", "status"]
for typo in ["comit", "chekout", "stauts"] {
  print("  {typo}? did you mean {typo.closest(commands)}")
}

# Anagram groups
words = "listen silent enlist stone tones notes onset rat tar art".words
print(words.group_by(|w| w.chars.sort.join).values)

# Encodings and hashes
print("base64: {"zil".base64}")
print("sha256: {"zil".sha256[..16]}...")
print("flipped: {"zil is fun".flip}")
print("pig latin: {"zil is fun".pig_latin}")
