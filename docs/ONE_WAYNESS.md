# Conditional average-case one-wayness

Let `R_base` be an average-case module-ISIS relation sampled from a distribution `D` over public statements. Let `Compile` be the deterministic Ogunedo/Keller wrapper and `Decode` recover the underlying short witness from every accepting compiled preimage.

For any probabilistic polynomial-time inverter `A` against the compiled relation, construct `B`:

1. receive a base challenge `S <- D`;
2. compute the compiled public statement `S' = Compile(S)`;
3. run `A(S')`;
4. if `A` returns an accepting compiled preimage, output `Decode` of that preimage.

When compilation and decoding are exact,

```text
Adv_ogunedo(A) <= Adv_module-ISIS(B)
                  + Delta_statement
                  + epsilon_soundness.
```

For the deterministic statement wrapper in this repository, `Delta_statement = 0`. For the exact relation checker, `epsilon_soundness` is inherited from the SP1 proof system rather than from a probabilistic relation test.

Therefore, conditional on average-case hardness of the selected module-ISIS distribution and SP1 soundness, the compiled relation is average-case one-way.

This is a preservation theorem. The compiler does not manufacture average-case hardness. A secure deployment must define a reviewed challenge/key distribution rather than using arbitrary hand-generated instances.
