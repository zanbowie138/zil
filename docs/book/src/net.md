# net

IP addresses, CIDR blocks, subnets

> Example results generated on 2026-10-06.
> Ones using `now`, `today` or randomness will differ when you run them.

## Functions

| function | description |
|---|---|
| [`ip(v)`](#ip) | an IPv4 or IPv6 address as an integer, or an integer as an address |
| [`cidr(s)`](#cidr) | an IPv4 block's network, broadcast, netmask, host range and count |
| [`in_cidr(ip, block)`](#in_cidr) | whether an IPv4 address is inside a CIDR block |
| [`subnets(block, prefix)`](#subnets) | split an IPv4 block into smaller blocks |
| [`ip_kind(ip)`](#ip_kind) | loopback, private, link-local, multicast, unspecified or public |

### ip

`ip(v)`: an IPv4 or IPv6 address as an integer, or an integer as an address

```zil
ip("192.168.1.5")
# → 3232235781
ip(3232235781)
# → "192.168.1.5"
ip("::1")
# → 1
```

See also: [cidr](net.md#cidr), [ip_kind](net.md#ip_kind)

### cidr

`cidr(s)`: an IPv4 block's network, broadcast, netmask, host range and count

```zil
cidr("192.168.1.77/26")
# → {network: "192.168.1.64", broadcast: "192.168.1.127", netmask: "255.255.255.192", prefix: 26, first: "192.168.1.65", last: "192.168.1.126", hosts: 62}
```

See also: [in_cidr](net.md#in_cidr), [subnets](net.md#subnets)

### in_cidr

`in_cidr(ip, block)`: whether an IPv4 address is inside a CIDR block

```zil
in_cidr("10.0.3.7", "10.0.0.0/22")
# → true
```

See also: [cidr](net.md#cidr)

### subnets

`subnets(block, prefix)`: split an IPv4 block into smaller blocks

```zil
subnets("10.0.0.0/24", 26)
# → ["10.0.0.0/26", "10.0.0.64/26", "10.0.0.128/26", "10.0.0.192/26"]
```

See also: [cidr](net.md#cidr)

### ip_kind

`ip_kind(ip)`: loopback, private, link-local, multicast, unspecified or public

```zil
ip_kind("10.1.2.3")
# → "private"
ip_kind("8.8.8.8")
# → "public"
```

See also: [ip](net.md#ip)

## More examples

### net

```zil
# hosts in a /22
cidr("10.0.0.0/22").hosts
# → 1022
# is it in the block?
in_cidr("10.0.3.7", "10.0.0.0/22")
# → true
# address as bits
ip("192.168.1.5") to bin
# → 0b11000000101010000000000100000101
# next address
ip(ip("10.0.0.255") + 1)
# → "10.0.1.0"
# split into /24s
subnets("10.0.0.0/22", 24)
# → ["10.0.0.0/24", "10.0.1.0/24", "10.0.2.0/24", "10.0.3.0/24"]
```
