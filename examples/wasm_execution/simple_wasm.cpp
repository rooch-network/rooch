#include <emscripten.h>
#include <string.h>

#ifdef __cplusplus
extern "C"{
#endif

EMSCRIPTEN_KEEPALIVE char * test_alloc(char *arg) {
    char append[] = "-append";
    size_t append_len = strlen(append);
    size_t arg_len = strlen(arg);
    char * buffer = (char *)malloc(arg_len + append_len + 1);
    memcpy(buffer, arg, arg_len);
    memcpy(buffer + arg_len, append, append_len);
    buffer[arg_len + append_len + 1] = '\0';
    return buffer;
}

#ifdef __cplusplus
}
#endif
