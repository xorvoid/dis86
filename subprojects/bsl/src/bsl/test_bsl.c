#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "bsl.h"

#define TEST_FAIL(...) do { fprintf(stderr, "TEST FAIL: "); fprintf(stderr, __VA_ARGS__); fprintf(stderr, "\n"); abort(); } while(0)

#define PARSE(s) ({ \
  bsl_t *b = bsl_parse_new(s, strlen(s), NULL); \
  if (!b) TEST_FAIL("'%s'", s); \
  b; })
#define CLEANUP(b) bsl_delete(b)

#define GET(b, k, v) get_helper(b, k, v, true)
#define GET_NODE(b, k) get_node_helper(b, k, true)
#define GET_FAIL(b, k) get_helper(b, k, "", false)

static inline void get_helper(bsl_t *b, const char *key, const char *exp_val, bool succeed)
{
  const char *val = bsl_get_str(b, key);
  if (succeed) {
    if (!val) TEST_FAIL("Failed to get string key: '%s'", key);
    if (0 != strcmp(val, exp_val)) TEST_FAIL("Mismatch value: expected '%s', got '%s'", exp_val, val);
  } else {
    if (val) TEST_FAIL("Expected failure, but got success on key: '%s'", key);
  }
}

static inline void get_node_helper(bsl_t *b, const char *key, bool succeed)
{
  bsl_t *val = bsl_get_node(b, key);
  if (succeed) {
    if (!val) TEST_FAIL("Failed to get node key: '%s'", key);
  } else {
    if (val) TEST_FAIL("Expected failure, but got success on key: '%s'", key);
  }
}

static void test_1(void)
{
  bsl_t *b = PARSE("foo bar");
  GET(b, "foo", "bar");
  GET_FAIL(b, "foo1");
  CLEANUP(b);
}

static void test_2(void)
{
  bsl_t *b = PARSE("foo bar good stuff   ");
  GET(b, "foo", "bar");
  GET(b, "good", "stuff");
  GET_FAIL(b, "foo1");
  CLEANUP(b);
}

static void test_3(void)
{
  bsl_t *b = PARSE("top {foo bar baz {} } top2 r ");
  GET(b, "top.foo", "bar");
  GET_FAIL(b, "top.foo.baz");
  GET_NODE(b, "top.baz");
  GET(b, "top2", "r");
  CLEANUP(b);
}

static void test_4(void)
{
  bsl_t *b = PARSE("top \"foo bar\" bot g quote \"{ key val }\"");
  GET(b, "top", "foo bar");
  GET(b, "bot", "g");
  GET(b, "quote", "{ key val }");
  CLEANUP(b);
}

int main(void)
{
  test_1();
  test_2();
  test_3();
  test_4();
}
