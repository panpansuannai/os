.altmacro
.macro SAVE_GP reg
    sd x\reg, \reg*8(sp)
.endm

.globl __alltraps
.globl trampoline
.align 2
__alltraps:
    csrrw sp, sscratch, sp
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they were saved on kernel stack
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it on the kernel stack
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # load kernel_satp into t0
    ld t0, 35*8(sp)
    # load trap_handler into t1
    ld t1, 37*8(sp)

    # copy context to stack
    # ld t0, 36*8(sp)
    # .set n, 0
    # .rept 38
    # ld t1, n*8(sp)
    # sd t1, n*8(t0)
    # .set n, n+1
    # .endr

    # move to kernel_sp
    ld sp, 36*8(sp)
    # switch to kernel space
    csrw satp, t0
    sfence.vma
    # jump to trap_handler
    jr t1

.macro LOAD_GP reg
    ld x\reg, \reg*8(sp)
.endm

.globl __restore
.globl __restore_end
__restore:
    # a0: *TrapContext in user space(Constant); a1: user space token
    # switch to user space
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    # now sp points to TrapContext in user space, start restoring based on it
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # restore general purpose registers except x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # back to user stack
    ld sp, 2*8(sp)
    sret
trampoline:
