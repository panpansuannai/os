  .section .text.user
  .globl hello
  .globl hello_end
hello:
  .incbin "/home/panpan/github/rcorelearn/os/src/user/00hello_world"
hello_end:
