export default function Page({ params }: { params: { address: string } }) {
  return (
    <>Hello, {params.address}</>
  );
}


