.syntax unified
.section .vectors
.thumb
    .org 0
vectors:
    .long empty_irq+1
    .long empty_irq+1
    .long empty_irq+1
    .long empty_irq+1
    .long empty_irq+1
    .long empty_irq+1
    .long empty_irq+1
    .long empty_irq+1
    .long empty_irq+1
    .long empty_irq+1
    .long empty_irq+1
    .long empty_irq+1
    .long empty_irq+1
    .long empty_irq+1
    .long empty_irq+1
    .long empty_irq+1

empty_irq:
    bx lr
