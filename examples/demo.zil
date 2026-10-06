# Data conversion demo: CSV in, JSON out
let csv = "name,age,city\nann,30,Oslo\nbob,17,Rome\ncy,42,Oslo\n"

let adults = parse_csv(csv)
  |> map(\r -> {name: r.name, age: int(r.age), city: r.city})
  |> filter(\r -> r.age >= 18)

let by_city = {}
for r in adults {
  if by_city[r.city] == nil { by_city[r.city] = [] }
  by_city[r.city].push(r.name)
}

print(to_json(by_city, true))
print("total age:", adults.map(\r -> r.age).reduce(0, \a, x -> a + x))
