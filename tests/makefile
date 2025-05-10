all:
	arm-none-eabi-as -c tests-vectors.s -o tests-vectors.o
	arm-none-eabi-gcc -mthumb -march=armv7-m -c tests.c -o tests.o
	arm-none-eabi-gcc -mthumb -march=armv7-m -nostartfiles -Tlink.ld tests-vectors.o tests.o -lm -o tests.elf
	arm-none-eabi-objcopy --strip-all --keep-symbols symbols.txt tests.elf tests.elf

	clang-18 --target=armv7em -mfloat-abi=hard -mthumb -c encode.s -o encode.o
	python3 parse.py > ../src/test_decoder.txt

clean:
	rm tests.o tests.elf
