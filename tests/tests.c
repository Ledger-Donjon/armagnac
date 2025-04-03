#include <memory.h>
#include <math.h>

const char* src = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";

int test_fibonacci(int n) {
  if (n == 0) {
    return 0;
  }
  if (n == 1) {
    return 1;
  }
  return test_fibonacci(n - 1) + test_fibonacci(n - 2);
}

float test_cos(float x) {
  return cosf(x);
}

float test_sqrt(float x) {
  return sqrtf(x);
}

size_t test_memcpy(char* dst) {
    size_t length = strlen(src);
    memcpy(dst, src, length + 1);
    return length;
}

float test_pow(float base, float exponent) {
  return pow(base, exponent);
}

float test_bkpt(float x) {
  float result = cos(x);
  __asm__("bkpt #165");
  return sin(result);
}
