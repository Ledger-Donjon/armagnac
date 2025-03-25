all:
	arm-none-eabi-gcc -mthumb -march=armv7-m -nostartfiles -Tlink.ld benchmark.c -o benchmark.o -lm
	arm-none-eabi-objcopy --strip-all --keep-symbols symbols.txt benchmark.o benchmark.elf

clean:
	rm benchmark.o benchmark.elf
