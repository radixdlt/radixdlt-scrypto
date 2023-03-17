#include <glib.h>
#include <glib/gstdio.h>
#include <gio/gio.h>
#include <gio/gunixsocketaddress.h>
#include <qemu-plugin.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <fcntl.h>

QEMU_PLUGIN_EXPORT int qemu_plugin_version = QEMU_PLUGIN_VERSION;

const gchar* SERVER_SOCKET_ADDR = "/tmp/scrypto-qemu-plugin-server.socket";

static guint64 instructions_count;
static GThread* thr;
static GMutex data_lock;
static GString* addr;
static gboolean count_instructions = true;
static bool logging = false;
static guint64* shared_mem_ptr = NULL;


void* create_shared_memory(size_t size) 
{
  // Our memory buffer will be readable and writable:
  //int protection = PROT_READ | PROT_WRITE;

  // The buffer will be shared (meaning other processes can access it), but
  // anonymous (meaning third-party processes cannot obtain an address for it),
  // so only this process and its children will be able to use it:
  //int visibility = MAP_SHARED | MAP_ANONYMOUS;

    int fd = shm_open( "/shm-radix", O_CREAT | O_EXCL | O_RDWR, S_IRUSR | S_IWUSR);
    if (fd == -1)
    {
        g_print("Error calling shm_open()\n");
        return NULL;
    }

    if (ftruncate(fd, sizeof(guint64)) == -1)
    {
        g_print("Error calling ftruncate()\n");
        return NULL;
    }

    return mmap(NULL, sizeof(guint64), PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);



  // The remaining parameters to `mmap()` are not important for this use case,
  // but the manpage for `mmap` explains their purpose.
  //return mmap(NULL, size, PROT_READ | PROT_WRITE, MAP_SHARED | MAP_ANONYMOUS, -1, 0);
}

static void vcpu_tb_trans_callback(qemu_plugin_id_t id, struct qemu_plugin_tb *tb)
{
    if( !count_instructions ) return;

    gsize n = qemu_plugin_tb_n_insns(tb);

    for ( gsize i = 0; i < n; ++i) 
    {
        struct qemu_plugin_insn *instructions = qemu_plugin_tb_get_insn(tb, i);

        //g_mutex_lock(&data_lock);

        qemu_plugin_register_vcpu_insn_exec_inline(instructions, QEMU_PLUGIN_INLINE_ADD_U64, &instructions_count, 1);
//        qemu_plugin_register_vcpu_insn_exec_inline(instructions, QEMU_PLUGIN_INLINE_ADD_U64, &instructions_count, 1);

        //g_mutex_unlock(&data_lock);
    }

    *shared_mem_ptr = instructions_count;
}

gpointer thr_callback(gpointer data)
{
    g_print("Thread started!\n");

    GError* error = NULL;

    g_unlink(addr->str);

    GSocket *socket = g_socket_new (G_SOCKET_FAMILY_UNIX, G_SOCKET_TYPE_DATAGRAM, 0, &error);

    if (!socket)
    {
      g_printerr ("Error creating Unix socket: %s\n", error->message);
      return NULL;
    }

    GSocketAddress *src_address = g_unix_socket_address_new(addr->str);
    if (!src_address)
    {
      g_printerr ("Wrong Unix socket address: %s\n", addr->str);
      return NULL;
    }

    if( !g_socket_bind(socket, src_address, TRUE, &error))
    {
      g_printerr ("Can't bind socket: %s\n", error->message);
      return NULL;
    }

    g_object_unref(src_address);


    gchar buffer[4096];
    gssize size;
    gsize to_send;
    GSocketAddress *address = NULL;

    while(TRUE)
    {
        if (logging) g_print("waiting for data...");

        size = g_socket_receive_from(socket, &address, buffer, sizeof buffer, NULL, &error);

        if(size < 0)
        {
            g_printerr ("Error receiving from socket: %s\n", error->message);
            return NULL;
        }

        GUnixSocketAddress *uaddr = G_UNIX_SOCKET_ADDRESS (address);

        if (logging) 
        {
            g_print ("received %" G_GSSIZE_FORMAT " bytes of data", size);
            g_print (" from %s type %d", g_unix_socket_address_get_path(uaddr), g_unix_socket_address_get_address_type (uaddr));
            g_print ("\n-------------------------\n%.*s\n-------------------------\n",(int)size, buffer);
        }

        if( g_unix_socket_address_get_address_type(uaddr) != 2 )
        {
            if (logging) g_print("Only path address type is supported\n");
            continue;
        }

        g_mutex_lock(&data_lock);
        guint64 cnt = instructions_count;
        g_mutex_unlock(&data_lock);
        guint64 out_data = GUINT64_TO_BE(cnt);

        to_send = sizeof(out_data);

        size = g_socket_send_to(socket, address, (gchar*)&out_data, to_send, NULL, &error);

        if (logging) g_print("sending data back... %" G_GUINT64_FORMAT "\n", cnt);

        if(size < 0)
        {
            g_printerr("Error sending to socket: %s\n", error->message);
            return NULL;
        }
    }

    if (!g_socket_close (socket, &error))
    {
        g_printerr ("Error closing master socket: %s\n", error->message);
        return NULL;
    }
    g_object_unref (socket);

    g_print("Thread end\n");
    return NULL;
}

static void plugin_exit(qemu_plugin_id_t id, void *p)
{
    shm_unlink("/shm-radix");
    //qemu_plugin_outs("some text");
}

QEMU_PLUGIN_EXPORT int qemu_plugin_install(qemu_plugin_id_t id,
                                           const qemu_info_t *info,
                                           int argc, char **argv)
{
    addr = g_string_new(SERVER_SOCKET_ADDR);

    for(gint i = 0; i < argc; ++i)
    {
        g_autofree char **tokens = g_strsplit(argv[0], "=", 2);
        if (g_strcmp0(tokens[0], "socket") == 0 && tokens[1]) 
        {
            addr = g_string_new(tokens[1]);
        }
        else if (g_strcmp0(tokens[0], "log") == 0 &&
                 qemu_plugin_bool_parse(tokens[0], tokens[1], &logging)) 
        { // ok
        }
        else
        {
            fprintf(stderr, "bad parameters: %s\n", argv[0]);
            return -1;
        }
    }

    shared_mem_ptr = (guint64*)create_shared_memory(sizeof(guint64));
    if(shared_mem_ptr)
    {
        g_print("Shared memory allocated\n");
        *shared_mem_ptr = 0;
    }
    else
    {
        g_print("Shared memory allocation error\n");
        return -1;
    }

    g_print("Using socket path: %s\n", addr->str);

    thr = g_thread_new("Unix socket service", thr_callback, NULL);

    qemu_plugin_register_vcpu_tb_trans_cb(id, vcpu_tb_trans_callback);
    qemu_plugin_register_atexit_cb(id, plugin_exit, NULL);

    return 0;
}
