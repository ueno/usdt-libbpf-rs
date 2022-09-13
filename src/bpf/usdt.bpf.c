#include "vmlinux.h"
#include <bpf/usdt.bpf.h>

struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 4096 /* one page */);
} ringbuf SEC(".maps");

static __always_inline int
record_first_arg (struct pt_regs *ctx)
{
  uint64_t *result = bpf_ringbuf_reserve (&ringbuf, sizeof(uint64_t), 0);
  if (!result)
    return 1;

  long arg;
  bpf_usdt_arg (ctx, 0, &arg);

  *result = arg;
  bpf_ringbuf_submit (result, 0);

  return 0;
}

SEC("usdt//home/ueno/devel/tests/dt/hello_usdt:provider:function")
int BPF_USDT(usdt__trace) {
	return record_first_arg(ctx);
}

char LICENSE[] SEC("license") = "GPL";
