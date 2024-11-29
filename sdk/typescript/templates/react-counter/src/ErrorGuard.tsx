import { useError } from "@roochnetwork/rooch-sdk-kit";
import { Button } from "@radix-ui/themes";

export function ErrorGuard() {
  const { error, resolve } = useError();
  console.log(error);
  return error ? <Button onClick={resolve}>resolve</Button> : <></>;
}
