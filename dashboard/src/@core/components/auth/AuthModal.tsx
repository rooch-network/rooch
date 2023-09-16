// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import React, { useState } from 'react'
import {
  Modal,
  TextField,
  FormControlLabel,
  Checkbox,
  RadioGroup,
  Radio,
  Button,
} from '@mui/material'

interface Props {
  onSuccess: (key: string) => void;
}

const AuthModal: React.FC<Props> = ({ onSuccess }) => {
  const [loading, setLoading] = useState(false);

  const onFinish = async (values: any) => {
    setLoading(false);
  };

  return (
    <Modal open onClose={() => { }}>
      <div
        style={{
          position: 'absolute' as 'absolute',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          width: 400,
          bgcolor: 'background.paper',
          border: '2px solid #000',
          boxShadow: 24,
          p: 4,
        }}
      >
        <TextField
          autoFocus
          margin="dense"
          label="账号"
          fullWidth
          variant="standard"
        />

        <FormControlLabel
          control={<Checkbox />}
          label="读数据"
        />

        <RadioGroup>
          <FormControlLabel
            value="1"
            control={<Radio />}
            label="1小时"
          />
        </RadioGroup>

        <Button variant="contained" onClick={() => { }}>
          授权
        </Button>
      </div>
    </Modal>
  );
};

export default AuthModal;