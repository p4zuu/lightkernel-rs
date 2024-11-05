# lightkernel

A light kernel I use to debug kvm. The idea is to debug a kvm-enabled vm (L1) on
the host (L0), in which I run this light kernel in a nested qemu+kvm vm (L2).

## Inspiration

Most of the kernel setup comes from:

- https://os.phil-opp.com/
