#include <ctype.h>
#include <fcntl.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

// The Most Desired Episode of Functional Programming

const int DEBUG = 0;

struct jesd84b51 {
  uint16_t id;
  char name[UINT8_MAX];
  uint16_t cell_num[2];
  uint8_t cell_value[UINT8_MAX];
  uint8_t cell_value_len;
};

int run(const char *const binary_path, const char *const config_path);
void printd(const char *format, ...);
int _get_binary_array(const char *const path, uint8_t *const buffer,
                      uint16_t len);
int _read_configuration(const char *const path, struct jesd84b51 *const j_array,
                        uint8_t *const j_len);
int _column_parser(char *const str, const char *const token,
                   struct jesd84b51 *const j);
void _print_jesd84b51(const struct jesd84b51 *j);
int _fill_cell_value(struct jesd84b51 *const j, uint8_t const j_len,
                     uint8_t *const buffer);

size_t LENGTH = 1024 / 2;

int main(int argc, char *argv[]) {
  if (argc != 3) {
    fprintf(stderr, "Usage: %s <input_file> <output_file>\n", argv[0]);
    return 1;
  }

  const char *binary_path = argv[1];
  const char *config_path = argv[2];
  if (run(binary_path, config_path))
    return 1;
  return 0;
}

int run(const char *const binary_path, const char *const config_path) {
  uint8_t data_buffer[LENGTH];

  if (_get_binary_array(binary_path, data_buffer, LENGTH))
    return 1;
  struct jesd84b51 j_arr[UINT8_MAX];
  uint8_t j_len = 0;
  if (_read_configuration(config_path, j_arr, &j_len))
    return 1;
  _fill_cell_value(j_arr, j_len, data_buffer);

  printd("buffer\n");
  for (int i = 0; i < LENGTH; i++) {
    printd("%d", data_buffer[i]);
  };
  printd("\n");
  for (uint16_t i = 0; i < j_len; i++) {
    _print_jesd84b51(&j_arr[i]);
  }
  return 0;
};

void printd(const char *format, ...) {
  if (DEBUG) {
    va_list args;
    va_start(args, format);
    vprintf(format, args);
    va_end(args);
  }
}

int _get_binary_array(const char *const path, uint8_t *const buffer,
                      uint16_t len) {
  const size_t FILE_LENGTH = LENGTH * 2;
  const int fd = open(path, O_RDONLY);
  if (fd == -1) {
    perror("Failed to open file");
    return 1;
  }
  uint8_t tmp_buffer[FILE_LENGTH];
  const size_t fact_length = read(fd, tmp_buffer, FILE_LENGTH);
  printd("Fact read length in %s :%d\n", path, fact_length);
  if (fact_length == -1) {
    perror("Failed to read file");
    return 1;
  }

  printd("Row data:\n");
  for (int i = 0; i < FILE_LENGTH; i++) {
    tmp_buffer[i] = toupper(tmp_buffer[i]);
    uint8_t num = tmp_buffer[i];
    uint8_t offset = num <= 57 ? 48 : 55; // Converting Ascii to hexadecimal
    tmp_buffer[i] -= offset;
    printd("%X", tmp_buffer[i]);
  }
  // clang-format off
  // "A" "B" -> "00000110" "00000111" 4 shift left-> "01100000" "00000111" Add them-> "01100111" -> A byte data: "AB"
  // clang-format on
  for (int i = 0; i < LENGTH * 2; i++) {
    if (i % 2 == 0) {
      tmp_buffer[i] = tmp_buffer[i] << 4;
    } else {
      tmp_buffer[i] = tmp_buffer[i] + tmp_buffer[i - 1];
    }
  }
  printd("\nModified data:\n");
  for (int i = 1, j = 0; i < FILE_LENGTH; i += 2) {
    buffer[j] = tmp_buffer[i];
    // printd("%d:%d\n", j, buffer[j]);
    j++;
  }
  close(fd);
  return 0;
};

int _read_configuration(const char *const path, struct jesd84b51 *const j_array,
                        uint8_t *const j_len) {
  const int fd = open(path, O_RDONLY);
  if (fd == -1) {
    perror("Failed to open file");
    return 1;
  };
  const off_t file_size = lseek(fd, 0, SEEK_END);
  printd("\n%s size: %d \n", path, file_size);
  if (file_size < 0) {
    perror("Error determining file size");
    close(fd);
    return 1;
  }
  lseek(fd, 0, SEEK_SET);

  char buffer[file_size];
  const ssize_t buf_len = read(fd, buffer, file_size);
  if (buf_len < 0) {
    perror("Error reading file");
    close(fd);
    exit(EXIT_FAILURE);
  }
  // clang-format off
  // 66CCFF 66CCFF -> {66CCFF 66CCFF} {EE0000 EE0000} -> {[66CCFF] [66CCFF]} {[EE0000] [EE0000]}
  // EE0000 EE0000
  // clang-format on
  uint16_t j_array_index = 0;
  {
    char row_buf[UINT16_MAX];
    uint8_t row_cursor = 0;
    for (uint16_t i = 0; i < buf_len; i++) {
      if (buffer[i] != '\n') {
        row_buf[row_cursor] = buffer[i];
        row_cursor++;
      } else { // Getting a line of data
        // for (int i = 0; i < row_cursor; i++) {
        //   printd("%c", row_buf[i]);
        // }
        struct jesd84b51 j;
        memset(&j, 0, sizeof(j));
        _column_parser(row_buf, ",", &j);
        j_array[j_array_index] = j;
        j_array_index++;
        row_cursor = 0;
        memset(&row_buf, 0, sizeof(row_buf));
      }
    }
  }
  printd("%d rows\n", j_array_index);
  *j_len = j_array_index;
  return 0;
};

int _column_parser(char *const str, const char *const token,
                   struct jesd84b51 *const j) {
  // printd("%s\n", str);
  j->id = strtol(strtok(str, token), NULL, 10); // TODO: Need error handling
  char *name = strtok(NULL, token);
  stpncpy(j->name, name, sizeof(j->name) - 1);
  char *cell_num_prev = strtok(NULL, token);
  j->cell_num[0] = strtol(cell_num_prev, NULL, 10);
  char *cell_num_next = strtok(NULL, token);
  j->cell_num[1] = strtol(cell_num_next, NULL, 10);
  return 0;
};

int _fill_cell_value(struct jesd84b51 *const j, uint8_t const j_len,
                     uint8_t *const buffer) {
  for (uint8_t i = 0; i < j_len; i++) {
    uint8_t m = 0;
    if (j[i].cell_num[1] != UINT16_MAX) { // Have cell range
      for (uint16_t k = j[i].cell_num[1]; k <= j[i].cell_num[0]; k++) {
        j[i].cell_value[m] = buffer[k];
        m++;
      }
      j[i].cell_value_len = m;
    } else { // Only one cell
      j[i].cell_value[0] = buffer[j[i].cell_num[0]];
      j[i].cell_value_len = 1;
    }
  }
  return 0;
};

void _print_jesd84b51(const struct jesd84b51 *j) {
  printf("ID: %u\n", j->id);

  if (strlen(j->name) != 0) {
    printf("Name: %s\n", j->name);
  } else {
    printf("Name: NULL\n");
  }

  if (j->cell_num[1] != UINT16_MAX) {
    printf("Cell Numbers: [%u, %u]\n", j->cell_num[0], j->cell_num[1]);
  } else {
    printf("Cell Numbers: [%u]\n", j->cell_num[0]);
  }

  if (j->cell_value_len > 0) {
    printf("Cell Values: [");
    for (size_t i = 0; i < j->cell_value_len; i++) {
      printf("%u", j->cell_value[i]);
      if (i < j->cell_value_len - 1) {
        printf(", ");
      }
    }
    printf("]\n");
  } else {
    printf("Cell Values: NULL or empty\n");
  }
}