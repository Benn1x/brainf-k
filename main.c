extern void exec(char *ptr);
int main() {
  char* array = calloc(30000, sizeof(char));
  exec(array);
  return 0;
}
