
/* Hydra function type */
#define HYDRA_FUNC(name) hydra_result_t name(hydra_machine_t *m)

/* Hydra result: How to resume the x86-16 emulator? */
enum {
  HYDRA_RESULT_TYPE_RESUME,
  HYDRA_RESULT_TYPE_JUMP,
  HYDRA_RESULT_TYPE_JUMP_NEAR,
  HYDRA_RESULT_TYPE_CALL,
  HYDRA_RESULT_TYPE_CALL_NEAR,
  HYDRA_RESULT_TYPE_RET_NEAR,
  HYDRA_RESULT_TYPE_RET_FAR,
};

typedef struct hydra_result hydra_result_t;
struct hydra_result
{
  int      type;    /* HYDRA_RESULT_TYPE_* */
  uint16_t new_cs;
  uint16_t new_ip;
};

#define HYDRA_RESULT_RESUME()       ({ hydra_result_t res = {HYDRA_RESULT_TYPE_RESUME, -1, -1}; res; })
#define HYDRA_RESULT_JUMP(seg, off) ({ hydra_result_t res = {HYDRA_RESULT_TYPE_JUMP, seg, off}; res; })
#define HYDRA_RESULT_JUMP_NEAR(off) ({ hydra_result_t res = {HYDRA_RESULT_TYPE_JUMP_NEAR, 0, off}; res; })
#define HYDRA_RESULT_RET_NEAR()     ({ hydra_result_t res = {HYDRA_RESULT_TYPE_RET_NEAR, -1, -1}; res; })
#define HYDRA_RESULT_RET_FAR()      ({ hydra_result_t res = {HYDRA_RESULT_TYPE_RET_FAR, -1, -1}; res; })

/* Registration flags */
enum {
  HYDRA_HOOK_FLAGS_OVERLAY = 1<<0,
};

/* Registration macros */
#define HYDRA_REGISTER_ADDR(func, seg, off, flags) hydra_impl_register_addr(func, seg, off, flags)
#define HYDRA_REGISTER(name) hydra_impl_register("F_" #name, H_ ## name, IS_OVERLAY_ENTRY_F_ ## name)
#define HYDRA_DEAD_ADDR(func, seg, off, flags) HYDRA_REGISTER_ADDR(hydra_impl_dead, seg, off, flags)
#define HYDRA_DEAD(name, flags) hydra_impl_register("F_" #name, hydra_impl_dead, flags)

/* Implementations provided "out-of-line" */
void           hydra_impl_register_addr(hydra_result_t (*func)(hydra_machine_t *m), u16 seg, u16 off, int flags);
void           hydra_impl_register(const char *name, hydra_result_t (*func)(hydra_machine_t *m), int flags);
hydra_result_t hydra_impl_dead(hydra_machine_t *m);
