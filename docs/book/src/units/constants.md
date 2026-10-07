# units.constants

physical constants (CODATA 2018) as quantities; `h` is still hours, so Planck's constant is `h_planck`

> Example results generated on 2026-10-07.
> Ones using `now`, `today` or randomness will differ when you run them.

### constants

```zil
# speed of light
c
# → 299792458 m/s
# gravitational constant
G
# → 6.6743e-11 m^3/kg/s^2
# Planck constant
h_planck
# → 6.62607e-34 J*s
# reduced Planck constant
hbar
# → 1.05457e-34 J*s
# Boltzmann constant
k_B
# → 1.38065e-23 J/K
# Avogadro constant
N_A
# → 6.02214e23 1/mol
# gas constant
R
# → 8.31446 J/mol/K
# elementary charge
e_charge
# → 1.60218e-19 coulomb
# electron mass
m_e
# → 9.10938e-31 kg
# proton mass
m_p
# → 1.67262e-27 kg
# standard gravity
g0
# → 9.80665 m/s^2
# vacuum permittivity
eps0
# → 8.85419e-12 farad/m
# vacuum permeability
mu0
# → 0.00000125664 N/A^2
# Stefan-Boltzmann constant
sigma
# → 5.67037e-8 W/m^2/K^4
```

## More examples

### constants

```zil
# mass-energy
1 kg * c**2 to J
# → 8.98755e16 J
# light from the sun
1 au / c to min
# → 8.31675 min
# photon energy of green light
h_planck * c / 530 nm to eV
# → 2.33932 eV
# weight
70 kg * g0
# → 686.465 N
```
