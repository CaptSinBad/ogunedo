# Parameter policy

## Registry rule

The guest accepts only parameter sets compiled into `ogunedo-core`. Each entry contains:

- immutable identifier and name;
- modulus and ring degree;
- module dimensions;
- coefficient and squared-norm bounds;
- primitive generator;
- lifecycle status.

## Status values

- `DevelopmentOnly`: correctness and test fixtures only;
- `CryptanalysisRequired`: engineering candidate, not approved for security claims;
- `ProductionApproved`: independently reviewed and authorized for deployment.

No `ProductionApproved` profile exists in v0.1.0.

## Promotion requirements

A profile must not be promoted until all of the following are complete:

1. exact SIS/module-SIS estimator inputs are published;
2. classical and quantum attack estimates are independently reproduced;
3. algebraic and subfield/module-structure attacks are reviewed;
4. witness distribution and target distribution are specified;
5. trapdoor generation and sampling, when used, are independently audited;
6. proof cost and denial-of-service limits are benchmarked;
7. two independent implementations agree on test vectors;
8. an external cryptographic review signs off on the profile.

Changing any numeric parameter requires a new identifier.
