// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

import { render, fireEvent, waitFor } from '@testing-library/react';
import AuthModal from './AuthModal';

test('submits form and calls onSuccess', async () => {
  const onSuccess = jest.fn();

  const { getByPlaceholderText, getByLabelText, getByText } = render(
    <AuthModal onSuccess={onSuccess} />
  );

  fireEvent.change(getByPlaceholderText('账号'), { 
    target: { value: 'test' }
  });

  fireEvent.click(getByLabelText('读数据'));

  fireEvent.click(getByText('1小时'));

  fireEvent.click(getByText('授权'));

  await waitFor(() => {
    expect(onSuccess).toHaveBeenCalledWith('mockKey');
  });
});