#include "ruby_ext.h"

int rb_float_type_p(VALUE obj)
{
    return RB_FLOAT_TYPE_P(obj);
}
