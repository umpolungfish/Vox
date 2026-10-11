.global _start
.section .text
_start:
    mov $1, %eax
    mov $1, %edi
    lea word(%rip), %rsi
    mov $word_end-word, %edx
    syscall
    mov $60, %eax
    mov $status, %edi
    syscall
.section .rodata
word:
    .incbin "measurements/modular_residue_20261010/stdout_word.imasm"
word_end:
