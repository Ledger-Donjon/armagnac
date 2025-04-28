all:
	arm-none-eabi-gcc -mthumb -march=armv7-m -nostartfiles -Tlink.ld tests.c -o tests.o -lm
	arm-none-eabi-objcopy --strip-all --keep-symbols symbols.txt tests.o tests.elf
	clang-18 --target=armv7em -mfloat-abi=hard -mthumb -c encode.s -o encode.o
	python3 parse.py > ../src/test_decoder.txt

clean:
	rm tests.o tests.elf
