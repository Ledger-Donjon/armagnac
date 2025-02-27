all:
	arm-none-eabi-gcc -mthumb -nostartfiles fibonacci.c -o fibonacci.o
	arm-none-eabi-objcopy fibonacci.o -O binary fibonacci.bin

clean:
	rm fibonacci.o fibonacci.bin
