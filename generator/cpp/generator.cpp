#include <emscripten.h>
#include "nlohmann/json.hpp"

using json = nlohmann::json;

#ifdef __cplusplus
extern "C"{
#endif

uint32_t hash_str_uint32(const std::string& str) {

    uint32_t hash = 0x811c9dc5;
    uint32_t prime = 0x1000193;

    for(int i = 0; i < str.size(); ++i) {
        uint8_t value = str[i];
        hash = hash ^ value;
        hash *= prime;
    }

    return hash;
}

uint32_t get_data_length(const char *buf) {
    char data_len_buf[4];
    data_len_buf[3] = buf[0];
    data_len_buf[2] = buf[1];
    data_len_buf[1] = buf[2];
    data_len_buf[0] = buf[3];
    uint32_t data_len;
    memcpy(&data_len, data_len_buf, 4);
    return data_len;
}

char * int_to_bytes(uint32_t n) {
    char* bytes = (char *)malloc(4);

    bytes[0] = (n >> 24) & 0xFF;
    bytes[1] = (n >> 16) & 0xFF;
    bytes[2] = (n >> 8) & 0xFF;
    bytes[3] = n & 0xFF;

    return bytes;
}

EMSCRIPTEN_KEEPALIVE const char * inscribe_generate(const char* buffer) {
    printf("inscribe_generate_start\n");

    uint32_t buffer_length = get_data_length(buffer);
    printf("buffer_length: %d\n", buffer_length);

    std::vector<uint8_t> buffer_vec;
    buffer_vec.insert(buffer_vec.end(), buffer + 4, buffer + 4 + buffer_length);
    json json_object_top = json::from_cbor(buffer_vec.begin(), buffer_vec.end());

    json json_output;

    std::string seed;
    json_object_top["seed"].get_to(seed);
    printf("seed: %s\n", seed.c_str());

    std::string user_input;
    json_object_top["user_input"].get_to(user_input);
    printf("user_input: %s\n", user_input.c_str());

    std::vector<uint8_t> attrs_buffer;
    json_object_top["attrs"].get_to(attrs_buffer);

    json attrs_object = json::from_cbor(attrs_buffer.begin(), attrs_buffer.end());
    uint32_t hash_value = hash_str_uint32(seed + user_input);

    if ((!attrs_object.empty()) && (attrs_object.is_array())) {
        for (json::iterator it = attrs_object.begin(); it != attrs_object.end(); ++it) {
            json attr = *it;
            if (attr.is_object()) {
                for (json::iterator it_inner = attr.begin(); it_inner != attr.end(); ++it_inner) {
                    json attr_value = it_inner.value();
                    if (attr_value.is_object()) {
                        std::string attr_key = it_inner.key();
                        if ((attr_value.contains("data")) && (attr_value.contains("type"))) {
                            std::string attr_type = attr_value["type"];
                            if (attr_type == "range") {
                                json attr_data = attr_value["data"];
                                uint32_t range_min = attr_data["min"];
                                uint32_t range_max = attr_data["max"];
                                uint32_t random_value = range_min + (hash_value % (range_max - range_min + 1));
                                json_output.emplace("id", user_input);
                                json_output.emplace(attr_key, random_value);
                            }
                        }
                    }
                }
            }
        }
    }

    json content(json::value_t::object);
    json top_json_output;
    top_json_output.emplace("amount", 1000);
    top_json_output.emplace("attributes", json_output);
    top_json_output.emplace("content", content);

    std::vector<std::uint8_t> dump = json::to_cbor(top_json_output);
    size_t dump_len = dump.size();
    char * length_bytes = int_to_bytes((uint32_t)dump_len);
    char * output = (char *)dump.data();
    char * buffer_output = (char *)malloc(sizeof(char) * dump_len + 4);
    memcpy(buffer_output, length_bytes, 4);
    memcpy(buffer_output + 4, output, dump_len);
    free(length_bytes);

    printf("inscribe_generate_end\n");

    return buffer_output;
}

EMSCRIPTEN_KEEPALIVE bool inscribe_verify(const char* buffer, const char* inscribe_output_buffer) {
    const char *inscribe_output = inscribe_generate(buffer);
    uint32_t output_len = get_data_length(inscribe_output);
    bool result = memcmp(inscribe_output+4, inscribe_output_buffer, output_len) == 0;
    free((void*)inscribe_output);  // Free the dynamically allocated memory
    return result;
}

EMSCRIPTEN_KEEPALIVE const char * indexer_generate(const char* buffer) {
    char seed[] = "";
    return inscribe_generate(buffer);
}

#ifdef __cplusplus
}
#endif
