module Main exposing (main)

import Browser
import Html exposing (Html, button, div, h1, span, text)
import Html.Attributes exposing (class)
import Html.Events exposing (onClick)


main : Program () Int Msg
main =
    Browser.sandbox { init = 0, update = update, view = view }


type Msg
    = Increment
    | Decrement
    | Reset


update : Msg -> Int -> Int
update msg model =
    case msg of
        Increment ->
            model + 1

        Decrement ->
            model - 1

        Reset ->
            0


view : Int -> Html Msg
view model =
    div [ class "p-3" ]
        [ h1 [ class "text-3xl font-bold" ] [ text "Hello, World!" ]
        , div [ class "grid min-h-screen place-items-center" ]
            [ span [ class "flex gap-3" ]
                [ button [ onClick Decrement, class "w-6 bg-black text-white rounded-full" ] [ text "-" ]
                , div [] [ text (String.fromInt model) ]
                , button [ onClick Increment, class "w-6 bg-black text-white rounded-full" ] [ text "+" ]
                , button [ onClick Reset, class "button rounded-full" ] [ text "🔥" ]
                ]
            ]
        ]
