CC ?= gcc
CXX ?= g++
CXXFLAGS ?=
AR ?= ar

STB_IMAGE_C_SRC=stb_image.c
STB_IMAGE_OBJS=$(STB_IMAGE_C_SRC:%.c=%.o)

CFLAGS += -fPIC

.PHONY: all
all:	${OUT_DIR}/libstb-image.a

%.o:	%.c
	$(CC) $< -o $@ -c $(CFLAGS)

${OUT_DIR}/libstb-image.a: $(STB_IMAGE_OBJS)
	$(AR) rcs $@ $^
