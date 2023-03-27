#include <glib.h>
#include <glib/gstdio.h>
#include <gio/gio.h>
#include <qemu-plugin.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <fcntl.h>

QEMU_PLUGIN_EXPORT int qemu_plugin_version = QEMU_PLUGIN_VERSION;

// Name of the shared memory resource
const gchar* SHARED_MEM_ID = "/shm-scrypto";

// Emulated instructions count
static guint64 instructions_count;
// Shared memory pointer
static guint64* shared_mem_ptr;


// Creates shared memory pointer.
void* create_shared_memory(size_t size) 
{
    int fd = shm_open( SHARED_MEM_ID, O_CREAT | O_TRUNC | O_RDWR, S_IRUSR | S_IWUSR);
    if (fd == -1)
    {
        qemu_plugin_outs("[QEMU-scrypto-plugin] Error calling shm_open()\n");
        return NULL;
    }

    if (ftruncate(fd, sizeof(guint64)) == -1)
    {
        qemu_plugin_outs("[QEMU-scrypto-plugin] Error calling ftruncate()\n");
        return NULL;
    }

    return mmap(NULL, sizeof(guint64), PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
}

// Emulated instruction executed callback - increasing counter and updating shared memory value.
void vcpu_udata_cb(unsigned int vcpu_index,void *userdata)
{
    instructions_count++;
    *shared_mem_ptr = instructions_count;
}

// Emulated instructions block transated callback - registering for each instruction execution callback.
static void vcpu_tb_trans_callback(qemu_plugin_id_t id, struct qemu_plugin_tb *tb)
{
    gsize n = qemu_plugin_tb_n_insns(tb);

    for ( gsize i = 0; i < n; ++i) 
    {
        struct qemu_plugin_insn *instructions = qemu_plugin_tb_get_insn(tb, i);

        qemu_plugin_register_vcpu_insn_exec_cb( instructions, vcpu_udata_cb, QEMU_PLUGIN_CB_NO_REGS, 0 );
    }
}

// Plugin cleanup
static void plugin_exit(qemu_plugin_id_t id, void *p)
{
    shm_unlink(SHARED_MEM_ID);
    qemu_plugin_outs("[QEMU-scrypto-plugin] Exit\n");
}

// Plugin entry function
QEMU_PLUGIN_EXPORT int qemu_plugin_install(qemu_plugin_id_t id,
                                           const qemu_info_t *info,
                                           int argc, char **argv)
{
    // Setup shared memory pointer
    shared_mem_ptr = (guint64*)create_shared_memory(sizeof(guint64));
    if(shared_mem_ptr)
    {
        qemu_plugin_outs("[QEMU-scrypto-plugin] Shared memory allocated\n");
        *shared_mem_ptr = 0;
    }
    else
    {
        qemu_plugin_outs("[QEMU-scrypto-plugin] Shared memory allocation error: ");
        qemu_plugin_outs(g_strerror(errno));
        qemu_plugin_outs("\n");
        return -1;
    }

    // Register qemu callbacks
    qemu_plugin_register_vcpu_tb_trans_cb(id, vcpu_tb_trans_callback);
    qemu_plugin_register_atexit_cb(id, plugin_exit, NULL);

    qemu_plugin_outs("[QEMU-scrypto-plugin] Started\n");
    return 0;
}
