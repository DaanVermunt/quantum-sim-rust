# Quantum assembler

## Operators

`INITIALIIZE R [INPUT]` Means set Register R to Byte array input e.g. `INITIALIZE R1 [0 0 0 0 1 1 1 1]`
`INITIALIIZE R [NR BITS]` Means set Register R to zero array of size NR BITSe.g. `INITIALIZE R2 4` -> `R2 = [0 0 0 0]`

`SELECT TO FROM START NUMQBITS` Means create sub register ref TO by selecting from FROM from START NUMQBIT e.g. `SELECT S1 R1 2 3` -> `S1 = [0 0 1]`

The basic gates are `G_H` (Hadamard), `G_R_2`, `G_R_4` (Phase shift pi/2 and pi/4), `G_I` (Identity), `G_CNOT` (controlled-NOT)

`APPLY U R` Means apply operator U to Register R e.g. `APPLY G_I R1` -> R1

`U3 CONCAT U1 U2` Create new operator U3 as a sequential operation of applying first U2 then U1

`U3 TENSOR U1 U2` Create new operator U3 as a tensor of U2 and U1
`U2 INVERSE U1` Create new operator U3 as a tensor of U2 and U1

`MEASURE R RES` Create new operator U3 as a tensor of U2 and U1