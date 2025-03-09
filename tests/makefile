all:
	arm-none-eabi-gcc -mthumb -nostartfiles fibonacci.c -o fibonacci.o
	arm-none-eabi-objcopy fibonacci.o -O binary fibonacci.bin
	arm-none-eabi-gcc -mthumb -nostartfiles -Tlink.ld memcpy.c -o memcpy.o
	arm-none-eabi-objcopy memcpy.o -O binary memcpy.bin

clean:
	rm fibonacci.o fibonacci.bin
