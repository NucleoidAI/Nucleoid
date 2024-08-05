#ifndef DATASTORE_H
#define DATASTORE_H

#include <stdbool.h>

typedef struct {
    bool cache;
} Config;

void init(Config config);

void clear(void);

Statement* read(void);

void write(Data data);

Statement* tail(int n);

#endif
