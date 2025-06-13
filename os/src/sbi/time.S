# set up a S-software interrupt here to delegate it to supervisor mode indirectly
# do not set S-timer interrupt since STIP can only be set by stime and stimecmp
    .section .text.time
    .globl __time_handler
    .align 2
__time_handler:
    csrrw sp, mscratch, sp
    sd t0, 0(sp)
    sd t1, 1*8(sp)
    sd t2, 2*8(sp)

    # setup next timer trigger
    ld t0, 3*8(sp) # address of mtimercmp
    ld t1, 4*8(sp) # timer interval
    ld t2, 0(t0) # current time
    add t2, t2, t1
    sd t2, 0(t0) # set newtime

    # setup timer interrupt for supervisor
    li t0, 2
    csrw sip, t0
    # csrrs zero, mip, t0

    # restore registers
    ld t0, 0(sp)
    ld t1, 1*8(sp)
    ld t2, 2*8(sp)
    csrrw sp, mscratch, sp

    mret