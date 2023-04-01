# Meson build system project definition
project('qemu scrypto plugin', 'c')

# update this path
qemu_source_dir = '/home/qemu/qemu-7.2.0/'

# to build this project run:
#  meson setup builddir
#  cd builddir
#  ninja
#
# required dependencies: meson, ninja, libglib2.0-dev


src = files('src/main.c')

inc = include_directories(join_paths(qemu_source_dir,'include/qemu'))

dep = [
    dependency('glib-2.0'),
    dependency('threads')
]

linkargs=''
if build_machine.system() == 'linux'
    linkargs = '-lrt'
endif

shared_module('scrypto-qemu-plugin', src,
    dependencies: dep,
    include_directories: inc,
    link_args: linkargs
)
