# Run: zil examples/dev.zil
# Networks, colors, versions and IDs.

net = cidr("192.168.4.0/22")
print(net)
print("as /24s: {subnets("192.168.4.0/22", 24)}")
for a in ["10.1.2.3", "8.8.8.8", "127.0.0.1"] { print("  {a.pad(10)} {ip_kind(a)}") }
print("port 5432 is {port(5432)}")

# Which text colors are readable on a dark blue background? (WCAG wants 4.5+)
bg = "#1e3a5f"
for fg in ["white", "#cccccc", "#666666"] {
  print("  {fg.pad(8)} {round(contrast(fg, bg), 2)}")
}
print("a gradient: {(0..5).map(|i| mix("#ff6b6b", "#4ecdc4", i / 4))}")

# The highest tag in each major version, in semver order
tags = ["1.9.3", "1.10.0", "2.0.0-rc.1", "2.0.0", "1.2.11", "2.1.0-beta"]
print(tags.group_by(|t| semver(t).major).map_values(|vs| vs.sort(semver).last))

print("uuid: {uuid()}")
