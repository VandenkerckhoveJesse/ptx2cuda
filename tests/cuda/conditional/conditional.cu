#include <stdio.h>

__device__ int g;

__global__ void conditional() {
    if (threadIdx.x == 0) g = 1;
}

int main() {
    conditional<<<1, 1>>>();
    return 0;
}