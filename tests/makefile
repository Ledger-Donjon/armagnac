all:
	arm-none-eabi-gcc -mthumb -march=armv7-m -nostartfiles -Tlink.ld tests.c -o tests.o -lm
	arm-none-eabi-objcopy --strip-all --keep-symbols symbols.txt tests.o tests.elf


clean:
	rm tests.o tests.elf
