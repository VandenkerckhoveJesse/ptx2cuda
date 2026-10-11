#include <stdio.h>

__global__ void noop() {
}

int main() {
    noop<<<1, 1>>>();
    return 0;
}