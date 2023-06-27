import Link from "next/link"

export const NotFound = ({ }) => (
    <div className="flex items-center justify-center min-h-screen py-48">
        <div className="flex flex-col">
            <div className="flex flex-col items-center">
                <div className="text-indigo-500 font-bold text-7xl">
                    404
                </div>

                <div className="font-bold text-3xl xl:text-7xl lg:text-6xl md:text-5xl mt-10">
                    This page does not exist
                </div>

                <div className="text-gray-400 font-medium text-sm md:text-xl lg:text-2xl mt-8">
                    The page you are looking for could not be found.
                </div>


                <Link href="/" className=" mt-10 font-bold text-sm md:text-lg lg:text-xl group-hover:underline 
                    text-gray-400 group-hover:text-gray-500
                    transition-all duration-200 delay-100" >
                    Home
                </Link>
            </div>
        </div>
    </div>
)