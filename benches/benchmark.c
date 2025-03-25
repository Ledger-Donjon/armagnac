#include <memory.h>
#include <math.h>

float bench_math(float x) {
  for (size_t i = 0; i < 10000; ++i) {
    x = cosf(sqrtf(x) * sqrtf(x)) + 1.0;
  }
  return x;
}