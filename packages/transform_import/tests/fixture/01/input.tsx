import { type FC } from "react";
import { useState } from "react";
import type { TablePaginationConfig } from "antd";
import {
    Button,
    Flex,
    PageHeader,
    Table as TableAntd,
    Popconfirm,
    message,
    type CheckboxValueType,
} from "antd";
import { fp } from "lodash";
import array from "lodash/array";
