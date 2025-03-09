#include <memory.h>

const char* src = "Lorem ipsum dolor sit amet, consectetur adipiscing elit.";

int main(char* dst) {
    size_t length = strlen(src);
    memcpy(dst, src, length + 1);
}
