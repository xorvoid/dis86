
static const char *as_upper(const char *s)
{
  static char buf[256];

  size_t len = strlen(s);
  if (len+1 >= sizeof(buf)) FAIL("String too long!");

  for (size_t i = 0; i < len+1; i++) {
    char c = s[i];
    if ('a' <= c && c <= 'z') c += ('A' - 'a');
    buf[i] = c;
  }

  return buf;
}
