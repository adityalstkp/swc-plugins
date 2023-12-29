import { type FC } from "react";
import { useState } from "react";
import type { TablePaginationConfig } from "antd";
import Button from "antd/lib/button";
import "antd/lib/button/style";
import Flex from "antd/lib/flex";
import "antd/lib/flex/style";
import PageHeader from "antd/lib/page-header";
import "antd/lib/page-header/style";
import TableAntd from "antd/lib/table";
import "antd/lib/table/style";
import Popconfirm from "antd/lib/popconfirm";
import "antd/lib/popconfirm/style";
import message from "antd/lib/message";
import "antd/lib/message/style";
import type { CheckboxValueType } from "antd";
import fp from "lodash/fp";
import array from "lodash/array";
